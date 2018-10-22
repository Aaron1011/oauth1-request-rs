extern crate either;

use self::either::Either;

use super::*;

macro_rules! delegate {
    (fn $method:ident(&mut self $(, $arg:ident: $typ:ty)*) $(-> $ret:ty)*; $($rest:tt)*) => {
        fn $method(&mut self $(, $arg: $typ)*) $(-> $ret)* {
            delegate! { @body $method(self.as_mut(), $($arg),*); }
        }
        delegate! { $($rest)* }
    };
    (fn $method:ident(&self $(, $arg:ident: $typ:ty)*) $(-> $ret:ty)*; $($rest:tt)*) => {
        fn $method(&self $(, $arg: $typ)*) $(-> $ret)* {
            delegate! { @body $method(self.as_ref(), $($arg),*); }
        }
        delegate! { $($rest)* }
    };
    (@body $method:ident($this:expr, $($arg:ident),*);) => {
        $this.either_with(
            ($($arg,)*),
            |($($arg,)*), l| l.$method($($arg,)*),
            |($($arg,)*), r| r.$method($($arg,)*),
        )
    };
    () => {};
}

impl<L: SignatureMethod, R: SignatureMethod> SignatureMethod for Either<L, R> {
    type Sign = Either<L::Sign, R::Sign>;

    delegate! {
        fn name(&self) -> &'static str;
        fn use_nonce(&self) -> bool;
        fn use_timestamp(&self) -> bool;
    }
}

impl<L: Sign, R: Sign> Sign for Either<L, R> {
    type SignatureMethod = Either<L::SignatureMethod, R::SignatureMethod>;
    type SignatureMethodRef<'a> = Either<&'a L::SignatureMethod, &'a R::SignatureMethod>;
    type Signature = Either<L::Signature, R::Signature>;

    fn new(
        consumer_secret: impl Display,
        token_secret: Option<impl Display>,
        signature_method: Self::SignatureMethod,
    ) -> Self {
        match signature_method {
            Either::Left(sm) => Either::Left(L::new(consumer_secret, token_secret, sm)),
            Either::Right(sm) => Either::Right(R::new(consumer_secret, token_secret, sm)),
        }
    }

    fn get_signature_method(&self) -> Self::SignatureMethodRef {
        self.as_ref()
    }

    delegate! {
        fn request_method(&mut self, method: &str);
        fn uri(&mut self, uri: impl Display);
        fn parameter(&mut self, key: &str, value: impl Display);
        fn delimit(&mut self);
    }

    fn finish(self) -> Self::Signature {
        self.map_left(L::finish).map_right(R::finish)
    }

    delegate! {
        fn callback(&mut self, default_value: &'static str, value: impl Display);
        fn nonce(&mut self, default_key: &'static str, value: impl Display);
        fn signature_method(&mut self, default_key: &'static str, default_value: &'static str);
        fn timestamp(&mut self, default_key: &'static str, value: u64);
        fn token(&mut self, default_key: &'static str, value: impl Display);
        fn verifier(&mut self, default_key: &'static str, value: impl Display);
        fn version(&mut self, default_key: &'static str, default_value: &'static str);
    }
}
