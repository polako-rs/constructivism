use std::marker::PhantomData;

macro_rules! construct {
    ($t:ty { $($f:ident: $e:expr,)+ }) => {
        {
            let fields = <$t>::construct_fields();
            let params = <<$t as Construct>::Params as IntoParams>::into_params();
            $(
                let param = fields.$f();
                let param = params
                    .extract_field(&param.field())
                    .define(param.field().into_field_value($e));
                let params = params + param;
            )+
            // let (params, _rest) = <$t>::split_params(params);
            let (params, _rest) = <Params<_> as ExtractValues<<$t as Construct>::Params>>::extract_values(params);
            <$t>::construct(params)
        }
        
    };
}

#[test]
fn test_main() {
    // ttt(name="asd".into(), age=23);
    let slider = construct!(Slider {
        max: 23.,
    });
    assert_eq!(slider, Slider { min: 0., max: 23., val: 0.});

    let slider_label = {
        let fields = <SliderLabel>::construct_fields();
        // let params = <<SliderLabel as Construct>::Params as IntoParams>::into_params();
        let params = <<SliderLabel as Construct>::WrappedParams as IntoParams>::into_params();
        let param = fields.value();
        let param = params
            .extract_field(&param.field())
            .define(param.field().into_field_value("Hello?"));
        let params = params + param;
        // let param = fields.val();
        // let param = params
        //     .extract_field(&param.field())
        //     .define(param.field().into_field_value(32.));
        // let params = params + param;
        let all_structs = SliderLabel::construct_all(params);
        // let (values, params) = <Params<_> as ExtractValues<<SliderLabel as Construct>::Params>>::extract_values(params);
        // <SliderLabel>::construct(values)
    };

}

// #[derive(Construct)]
#[derive(Debug, PartialEq)]
pub struct Slider {
    min: f32,
    max: f32,
    val: f32,
}

#[derive(Debug, PartialEq)]
pub struct SliderLabel {
    value: String
}

// output of derive:

impl Construct for Slider {
    type Fields = slider_construct::Fields;
    type Params = (slider_construct::min, slider_construct::max, slider_construct::val);
    type Wraps = ();
    type Wrapped = (Self, <Self::Wraps as Construct>::Wrapped);
    type WrappedParams = (
        slider_construct::min, slider_construct::max, slider_construct::val,
        <Self::Wraps as Construct>::WrappedParams
    );
    fn construct_fields() -> &'static Self::Fields {
        slider_construct::Fields::instance()
    }

    fn construct(params: Self::Params)-> Self {
        let (slider_construct::min(min), slider_construct::max(max), slider_construct::val(val)) = params;
        Self { min, max, val }
    }

    fn construct_all<P>(params: P) -> <Self as Construct>::Wrapped
    where 
        Self: Sized,
        P: ExtractValues<Self::Params, Output = <<<Self as Construct>::Wraps as Construct>::WrappedParams as IntoParams>::Target >,
        // O: ExtractValues<
        //     <<Self as Construct>::Wraps as Construct>::Params,
        //     Output = <<<Self as Construct>::Wraps as Construct>::WrappedParams as IntoParams>::Target
        // >
    {
        let (args, params) = params.extract_values();
        (Self::construct(args), ())
    }
}


mod slider_construct {
    use super::*;
    #[allow(non_camel_case_types)]
    #[derive(Default)]
    pub struct min(pub f32);


    #[allow(non_camel_case_types)]
    #[derive(Default)]
    pub struct max(pub f32);

    #[allow(non_camel_case_types)]
    #[derive(Default)]
    pub struct val(pub f32);

    impl IntoField for min {
        fn into_field() -> Field<Self> {
            Field(PhantomData)
        }
    }

    impl IntoField for max {
        fn into_field() -> Field<Self> {
            Field(PhantomData)
        }
    }
    impl IntoField for val {
        fn into_field() -> Field<Self> {
            Field(PhantomData)
        }
    }

    impl<T: Into<f32>> IntoFieldValue<T> for Field<min> {
        type Output = min;
        fn into_field_value(self, value: T) -> Self::Output {
            min(value.into())
        }    
    }

    impl<T: Into<f32>> IntoFieldValue<T> for Field<max> {
        type Output = max;
        fn into_field_value(self, value: T) -> Self::Output {
            max(value.into())
        }    
    }
    impl<T: Into<f32>> IntoFieldValue<T> for Field<val> {
        type Output = val;
        fn into_field_value(self, value: T) -> Self::Output {
            val(value.into())
        }    
    }

    pub struct Fields(PhantomData<<<Slider as Construct>::Wraps as Construct>::Fields>);
    impl Singleton for Fields {
        fn instance() -> &'static Self {
            &Fields(PhantomData)
        }
    }
    impl std::ops::Deref for Fields {
        type Target = <<Slider as Construct>::Wraps as Construct>::Fields;
        fn deref(&self) -> &Self::Target {
            <<<Slider as Construct>::Wraps as Construct>::Fields as Singleton>::instance()
        }
    }
    impl Fields {
        #[allow(unused)]
        pub fn min(&self) -> Param<min, f32> {
            Param::new()
        }
        #[allow(unused)]
        pub fn max(&self) -> Param<max, f32> {
            Param::new()
        }
        #[allow(unused)]
        pub fn val(&self) -> Param<val, f32> {
            Param::new()
        }
    }
}

impl Construct for SliderLabel {
    type Fields = slider_label_construct::Fields;
    type Params = (
        slider_label_construct::value,
        // <SliderLabel as Construct>::Params,
    );
    type Wraps = Slider;
    type Wrapped = (Self, <Self::Wraps as Construct>::Wrapped);
    type WrappedParams = (slider_label_construct::value, <Self::Wraps as Construct>::WrappedParams);
    fn construct_fields() -> &'static Self::Fields {
        slider_label_construct::Fields::instance()
    }

    fn construct(params: Self::Params)-> Self {
        let (slider_label_construct::value(value),) = params;
        Self { value }
    }
    fn construct_all<P>(params: P) -> <Self as Construct>::Wrapped
    where 
        Self: Sized,
        P: ExtractValues<Self::Params, Output = <<<Self as Construct>::Wraps as Construct>::WrappedParams as IntoParams>::Target >,
        // O: ExtractValues<
        //     <<Self as Construct>::Wraps as Construct>::Params,
        //     Output = <<<Self as Construct>::Wraps as Construct>::WrappedParams as IntoParams>::Target
        // >
    {
        let (args, params) = params.extract_values();
        (Self::construct(args), <<Self as Construct>::Wraps as Construct>::construct_all(params))
    }

    // fn construct_all<P, O>(params: P) -> <Self as Construct>::Wrapped
    //     where 
    //         Self: Sized,
    //         P: ExtractValues<Self::Params, Output = O>,
    //         O: ExtractValues<<<<Self as Construct>::Wraps as Construct>::Params as ExtractValues<
    //             <<Self as Construct>::Wraps as Construct>::Params
    //         >>::Output>
    // { 
    //     let (values, params) = P::extract_values(params);
    //     (Self::construct(values), <Self::Wraps as Construct>::construct_all(params))   
    // }
}

mod slider_label_construct {
    use super::*;
    #[allow(non_camel_case_types)]
    #[derive(Default)]
    pub struct value(pub String);


    impl IntoField for value {
        fn into_field() -> Field<Self> {
            Field(PhantomData)
        }
    }

    impl<T: Into<String>> IntoFieldValue<T> for Field<value> {
        type Output = value;
        fn into_field_value(self, v: T) -> Self::Output {
            value(v.into())
        }    
    }

    pub struct Fields(PhantomData<<<Slider as Construct>::Wraps as Construct>::Fields>);
    impl Singleton for Fields {
        fn instance() -> &'static Self {
            &Fields(PhantomData)
        }
    }
    impl std::ops::Deref for Fields {
        type Target = <<SliderLabel as Construct>::Wraps as Construct>::Fields;
        fn deref(&self) -> &Self::Target {
            <<<SliderLabel as Construct>::Wraps as Construct>::Fields as Singleton>::instance()
        }
    }
    impl Fields {
        #[allow(unused)]
        pub fn value(&self) -> Param<value, String> {
            Param::new()
        }
        
    }
}

// library expanded content for two types (I want 64 types at least)

// hand written:

pub trait Construct {
    type Fields: Singleton;
    type Params: IntoParams;
    type Wraps: Construct;
    type Wrapped;
    type WrappedParams: IntoParams;
    fn construct_fields() -> &'static Self::Fields;
    // fn construct_params() -> Params<<<Self as Construct>::UndefinedParams as IntoParams>::Target>;
    fn construct(params: Self::Params)-> Self;
    // fn split_params<T: SplitAt<{I}>>(params: T) -> (T::Left, T::Right){
    //     T::split(params)
    // }
    fn construct_all<P>(params: P) -> <Self as Construct>::Wrapped
    where 
        Self: Sized,
        P: ExtractValues<Self::Params, Output = <<<Self as Construct>::Wraps as Construct>::WrappedParams as IntoParams>::Target >;
}

impl Construct for () {
    type Fields = ();
    type Params = ();
    type Wraps = ();
    type Wrapped = ();
    type WrappedParams = ();
    fn construct_fields() -> &'static Self::Fields {
        &()
    }
    fn construct(_: Self::Params)-> Self {
        ()
    }
    fn construct_all<P>(_: P) -> <Self as Construct>::Wrapped
    where 
        Self: Sized,
        P: ExtractValues<Self::Params, Output = <<<Self as Construct>::Wraps as Construct>::WrappedParams as IntoParams>::Target >,
        // O: ExtractValues<
        //     <<Self as Construct>::Wraps as Construct>::Params,
        //     Output = <<<Self as Construct>::Wraps as Construct>::WrappedParams as IntoParams>::Target
        // >
    {
        ()
    }
    
    // fn construct_params() -> Params<<<Self as Construct>::UndefinedParams as IntoParams>::Target> {
    //     Params(())
    // }
}

// pub fn split<const I: usize, T:SplitAt<I>

pub struct Params<T>(T);
pub struct Param<T, V>(PhantomData<(T, V)>);
impl<T, V> Param<T, V> {
    fn new() -> Self {
        Self(PhantomData)
    }
    fn field(&self) -> Field<T> {
        Field(PhantomData)
    }
} 
pub struct Field<T>(PhantomData<T>);

pub struct D<const I: u8, T>(T);
pub struct U<const I: u8, T>(PhantomData<T>);
struct F<const I: u8, T>(PhantomData<T>);

trait A<const I: u8, T> { }

pub trait Singleton {
    fn instance() -> &'static Self;
}

impl Singleton for () {
    fn instance() -> &'static Self {
        &()
    }
}

trait ExtractField<F, T> {
    fn extract_field(&self, f: &Field<T>) -> F;
}

pub trait IntoField where Self: Sized {
    fn into_field() -> Field<Self>;
}

pub trait IntoFieldValue<T> {
    type Output;
    fn into_field_value(self, value: T) -> Self::Output;
}

pub trait MoveTo<const I: u8> {
    type Target;
    fn move_to(self) -> Self::Target;
}

pub trait ExtractValues<T> {
    type Output;
    fn extract_values(self) -> (T, Self::Output);
}


trait ExtractValue {
    type Value;
    fn extract_value(self) -> Self::Value;
}

impl<const I: u8, T> F<I, T> {
    fn define(self, value: T) -> D<I, T> {
        D::<I, T>(value)
    }
}


impl<const I: u8, T> A<I, T> for D<I, T> { }
impl<const I: u8, T> A<I, T> for U<I, T> { }

impl<const I: u8, const J: u8, T> MoveTo<J> for D<I, T> {
    type Target = D<J, T>;
    fn move_to(self) -> Self::Target {
        D::<J, T>(self.0)
    }
}
impl<const I: u8, const J: u8, T> MoveTo<J> for U<I, T> {
    type Target = U<J, T>;
    fn move_to(self) -> Self::Target {
        U::<J, T>(PhantomData)
    }
}
impl<const I: u8, const J: u8, T> MoveTo<J> for F<I, T> {
    type Target = F<J, T>;
    fn move_to(self) -> Self::Target {
        F::<J, T>(PhantomData)
    }
}

impl<const I: u8, T: Default> ExtractValue for U<I, T> {
    type Value = T;
    fn extract_value(self) -> T {
        T::default()
    }
}
impl<const I: u8, T> ExtractValue for D<I, T> {
    type Value = T;
    fn extract_value(self) -> T {
        self.0
    }
}

pub trait SplitAt<const I: u8> {
    type Left;
    type Right;
    fn split(self) -> (Self::Left, Self::Right);
}


impl<T> SplitAt<0> for Params<T> {
    type Left = ();
    type Right = Self;
    fn split(self) -> (Self::Left, Self::Right) {
        ((), self)
    }
}


// generated by macro

impl<T0, A0: A<0, T0>> ExtractField<F<0, T0>, T0> for Params<(A0,)> {
    fn extract_field(&self, _: &Field<T0>) -> F<0, T0> {
        F::<0, T0>(PhantomData)
    }
}
impl<T0, A0: A<0, T0>, A1> ExtractField<F<0, T0>, T0> for Params<(A0, A1)> {
    fn extract_field(&self, _: &Field<T0>) -> F<0, T0> {
        F::<0, T0>(PhantomData)
    }
}
impl<A0, A1: A<1, T1>, T1> ExtractField<F<1, T1>, T1> for Params<(A0, A1)> {
    fn extract_field(&self, _: &Field<T1>) -> F<1, T1> {
        F::<1, T1>(PhantomData)
    }
}
impl<T0, A0: A<0, T0>, A1, A2> ExtractField<F<0, T0>, T0> for Params<(A0, A1, A2)> {
    fn extract_field(&self, _: &Field<T0>) -> F<0, T0> {
        F::<0, T0>(PhantomData)
    }
}
impl<T1, A0, A1: A<1, T1>, A2> ExtractField<F<1, T1>, T1> for Params<(A0, A1, A2)> {
    fn extract_field(&self, _: &Field<T1>) -> F<1, T1> {
        F::<1, T1>(PhantomData)
    }
}
impl<T2, A0, A1, A2: A<2, T2>> ExtractField<F<2, T2>, T2> for Params<(A0, A1, A2)> {
    fn extract_field(&self, _: &Field<T2>) -> F<2, T2> {
        F::<2, T2>(PhantomData)
    }
}


impl<T0, A0: A<0, T0>, A1, A2, A3> ExtractField<F<0, T0>, T0> for Params<(A0, A1, A2, A3)> {
    fn extract_field(&self, _: &Field<T0>) -> F<0, T0> {
        F::<0, T0>(PhantomData)
    }
}
impl<T1, A0, A1: A<1, T1>, A2, A3> ExtractField<F<1, T1>, T1> for Params<(A0, A1, A2, A3)> {
    fn extract_field(&self, _: &Field<T1>) -> F<1, T1> {
        F::<1, T1>(PhantomData)
    }
}
impl<T2, A0, A1, A2: A<2, T2>, A3> ExtractField<F<2, T2>, T2> for Params<(A0, A1, A2, A3)> {
    fn extract_field(&self, _: &Field<T2>) -> F<2, T2> {
        F::<2, T2>(PhantomData)
    }
}
impl<T3, A0, A1, A2, A3: A<3, T3>> ExtractField<F<3, T3>, T3> for Params<(A0, A1, A2, A3)> {
    fn extract_field(&self, _: &Field<T3>) -> F<3, T3> {
        F::<3, T3>(PhantomData)
    }
}


impl<T0> std::ops::Add<D<0, T0>> for Params<(U<0, T0>,)> {
    type Output = Params<(D<0, T0>,)>;
    fn add(self, rhs: D<0, T0>) -> Self::Output {
        Params((rhs,))
    }
}
impl<T0, A1> std::ops::Add<D<0, T0>> for Params<(U<0, T0>, A1)> {
    type Output = Params<(D<0, T0>, A1)>;
    fn add(self, rhs: D<0, T0>) -> Self::Output {
        let (_, p1) = self.0;
        Params((rhs, p1))
    }
}
impl<A0, T1> std::ops::Add<D<1, T1>> for Params<(A0, U<1, T1>)> {
    type Output = Params<(A0, D<1, T1>)>;
    fn add(self, rhs: D<1, T1>) -> Self::Output {
        let (p0, _) = self.0;
        Params((p0, rhs))
    }
}
impl<T0, A1, A2> std::ops::Add<D<0, T0>> for Params<(U<0, T0>, A1, A2)> {
    type Output = Params<(D<0, T0>, A1, A2)>;
    fn add(self, rhs: D<0, T0>) -> Self::Output {
        let (_, p1, p2) = self.0;
        Params((rhs, p1, p2))
    }
}
impl<T1, A0, A2> std::ops::Add<D<1, T1>> for Params<(A0, U<1, T1>, A2)> {
    type Output = Params<(A0, D<1, T1>, A2)>;
    fn add(self, rhs: D<1, T1>) -> Self::Output {
        let (p0, _, p2) = self.0;
        Params((p0, rhs, p2))
    }
}
impl<T2, A0, A1> std::ops::Add<D<2, T2>> for Params<(A0, A1, U<2, T2>)> {
    type Output = Params<(A0, A1, D<2, T2>)>;
    fn add(self, rhs: D<2, T2>) -> Self::Output {
        let (p0, p1, _) = self.0;
        Params((p0, p1, rhs))
    }
}
impl<T0, A1, A2, A3> std::ops::Add<D<0, T0>> for Params<(U<0, T0>, A1, A2, A3)> {
    type Output = Params<(D<0, T0>, A1, A2, A3)>;
    fn add(self, rhs: D<0, T0>) -> Self::Output {
        let (_, p1, p2, p3) = self.0;
        Params((rhs, p1, p2, p3))
    }
}
impl<T1, A0, A2, A3> std::ops::Add<D<1, T1>> for Params<(A0, U<1, T1>, A2, A3)> {
    type Output = Params<(A0, D<1, T1>, A2, A3)>;
    fn add(self, rhs: D<1, T1>) -> Self::Output {
        let (p0, _, p2, p3) = self.0;
        Params((p0, rhs, p2, p3))
    }
}
impl<T2, A0, A1, A3> std::ops::Add<D<2, T2>> for Params<(A0, A1, U<2, T2>, A3)> {
    type Output = Params<(A0, A1, D<2, T2>, A3)>;
    fn add(self, rhs: D<2, T2>) -> Self::Output {
        let (p0, p1, _, p3) = self.0;
        Params((p0, p1, rhs, p3))
    }
}
impl<T3, A0, A1, A2> std::ops::Add<D<3, T3>> for Params<(A0, A1, A2, U<3, T3>)> {
    type Output = Params<(A0, A1, A2, D<3, T3>)>;
    fn add(self, rhs: D<3, T3>) -> Self::Output {
        let (p0, p1, p2, _) = self.0;
        Params((p0, p1, p2, rhs))
    }
}
impl<T> ExtractValues<()> for Params<T> {
    type Output = Self;
    fn extract_values(self) -> ((), Self::Output) {
        ((), self)
    }
}

impl<T0: ExtractValue<Value = P0>, P0> ExtractValues<(P0,)> for Params<(T0,)> {
    type Output = Params<()>;
    fn extract_values(self) -> ((P0,), Self::Output) {
        let (p0,) = self.0;
        ((
            p0.extract_value(),
        ), Params(()))
    }
}

impl ExtractValues<()> for () {
    type Output = Params<()>;
    fn extract_values(self) -> ((), Self::Output) {
        ((), Params(()))
    }
}

impl<T0, T1, P0> ExtractValues<(P0,)> for Params<(T0, T1)>
where
    T0: ExtractValue<Value = P0>,
    T1: MoveTo<0>,

{
    type Output = Params<(T1::Target,)>;
    fn extract_values(self) -> ((P0,), Self::Output) {
        let (p0, p1) = self.0;
        ((
            p0.extract_value(),
        ), Params((p1.move_to(),)))
    }
}
impl<T0, T1, T2, P0> ExtractValues<(P0,)> for Params<(T0, T1, T2)>
where
    T0: ExtractValue<Value = P0>,
    T1: MoveTo<0>,
    T2: MoveTo<1>,

{
    type Output = Params<(T1::Target, T2::Target)>;
    fn extract_values(self) -> ((P0,), Self::Output) {
        let (p0, p1, p2) = self.0;
        ((
            p0.extract_value(),
        ), Params((
            p1.move_to(),
            p2.move_to(),
        )))
    }
}
impl<T0, T1, T2, T3, P0> ExtractValues<(P0,)> for Params<(T0, T1, T2, T3)>
where
    T0: ExtractValue<Value = P0>,
    T1: MoveTo<0>,
    T2: MoveTo<1>,
    T3: MoveTo<2>,

{
    type Output = Params<(T1::Target, T2::Target, T3::Target)>;
    fn extract_values(self) -> ((P0,), Self::Output) {
        let (p0, p1, p2, p3) = self.0;
        ((
            p0.extract_value(),
        ), Params((
            p1.move_to(),
            p2.move_to(),
            p3.move_to(),
        )))
    }
}

impl<P0, P1, T0: ExtractValue<Value = P0>, T1: ExtractValue<Value = P1>> ExtractValues<(P0, P1)> for Params<(T0, T1)> {
    type Output = Params<()>;
    fn extract_values(self) -> ((P0, P1), Self::Output) {
        let (p0, p1) = self.0;
        ((
            p0.extract_value(),
            p1.extract_value(),
        ), Params(()))
    }
}
impl<P0, P1, T0, T1, T2> ExtractValues<(P0, P1)> for Params<(T0, T1, T2)>
where
    T0: ExtractValue<Value = P0>,
    T1: ExtractValue<Value = P1>,
    T2: MoveTo<0>,
{
    type Output = Params<(T2::Target,)>;
    fn extract_values(self) -> ((P0, P1), Self::Output) {
        let (p0, p1, p2) = self.0;
        ((
            p0.extract_value(),
            p1.extract_value(),
        ), Params((p2.move_to(),)))
    }
}
impl<P0, P1, T0, T1, T2, T3> ExtractValues<(P0, P1)> for Params<(T0, T1, T2, T3)>
where
    T0: ExtractValue<Value = P0>,
    T1: ExtractValue<Value = P1>,
    T2: MoveTo<0>,
    T3: MoveTo<1>,
{
    type Output = Params<(T2::Target, T3::Target)>;
    fn extract_values(self) -> ((P0, P1), Self::Output) {
        let (p0, p1, p2, p3) = self.0;
        ((
            p0.extract_value(),
            p1.extract_value(),
        ), Params((
            p2.move_to(),
            p3.move_to(),
        )))
    }
}
impl<P0, P1, P2, T0: ExtractValue<Value = P0>, T1: ExtractValue<Value = P1>, T2: ExtractValue<Value = P2>> ExtractValues<(P0, P1, P2)> for Params<(T0, T1, T2)> {
    type Output = Params<()>;
    fn extract_values(self) -> ((P0, P1, P2), Self::Output) {
        let (p0, p1, p2) = self.0;
        ((
            p0.extract_value(),
            p1.extract_value(),
            p2.extract_value(),
        ), Params(()))
    }
}
impl<P0, P1, P2, T0, T1, T2, T3> ExtractValues<(P0, P1, P2)> for Params<(T0, T1, T2, T3)>
where
    T0: ExtractValue<Value = P0>,
    T1: ExtractValue<Value = P1>,
    T2: ExtractValue<Value = P2>,
    T3: MoveTo<0>,
{
    type Output = Params<(T3::Target,)>;
    fn extract_values(self) -> ((P0, P1, P2), Self::Output) {
        let (p0, p1, p2, p3) = self.0;
        ((
            p0.extract_value(),
            p1.extract_value(),
            p2.extract_value(),
        ), Params((
            p3.move_to(),
        )))
    }
}
impl<P0, P1, P2, P3, T0, T1, T2, T3> ExtractValues<(P0, P1, P2, P3)> for Params<(T0, T1, T2, T3)>
where
    T0: ExtractValue<Value = P0>,
    T1: ExtractValue<Value = P1>,
    T2: ExtractValue<Value = P2>,
    T3: ExtractValue<Value = P3>,
{
    type Output = Params<()>;
    fn extract_values(self) -> ((P0, P1, P2, P3), Self::Output) {
        let (p0, p1, p2, p3) = self.0;
        ((
            p0.extract_value(),
            p1.extract_value(),
            p2.extract_value(),
            p3.extract_value(),
        ), Params(()))
    }
}


impl<T0, T1: MoveTo<0>> SplitAt<1> for Params<(T0, T1)> {
    type Left = Params<(T0,)>;
    type Right = Params<(T1::Target,)>;
    fn split(self) -> (Self::Left, Self::Right) {
        let (p0, p1) = self.0;
        (Params((p0,)), Params((p1.move_to(),)))
    }
}

impl<T0, T1: MoveTo<0>, T2: MoveTo<1>> SplitAt<1> for Params<(T0, T1, T2)> {
    type Left = Params<(T0,)>;
    type Right = Params<(T1::Target, T2::Target)>;
    fn split(self) -> (Self::Left, Self::Right) {
        let (p0, p1, p2) = self.0;
        (Params((p0,)), Params((p1.move_to(),p2.move_to())))
    }
}
impl<T0, T1> SplitAt<2> for Params<(T0, T1)> {
    type Left = Params<(T0, T1)>;
    type Right = Params<()>;
    fn split(self) -> (Self::Left, Self::Right) {
        let (p0, p1) = self.0;
        (Params((p0, p1,)), Params(()))
    }
}
impl<T0, T1, T2: MoveTo<0>> SplitAt<2> for Params<(T0, T1, T2)> {
    type Left = Params<(T0, T1)>;
    type Right = Params<(T2::Target,)>;
    fn split(self) -> (Self::Left, Self::Right) {
        let (p0, p1, p2) = self.0;
        (Params((p0,p1)), Params((p2.move_to(),)))
    }
}

impl<T0, T1, T2> SplitAt<3> for Params<(T0, T1, T2)> {
    type Left = Params<(T0, T1, T2)>;
    type Right = Params<()>;
    fn split(self) -> (Self::Left, Self::Right) {
        let (p0, p1, p2) = self.0;
        (Params((p0,p1,p2)), Params(()))
    }
}

pub trait IntoParams {
    type Target;
    fn into_params() -> Self::Target;
}
impl IntoParams for () {
    type Target = Params<()>;
    fn into_params() -> Self::Target {
        Params(())
    }
}
impl<T0: IntoField> IntoParams for (T0,) {
    type Target = Params<(U<0, T0>,)>;
    fn into_params() -> Self::Target {
        Params((U::<0, _>(PhantomData),))
    }
}
impl<T0: IntoField> IntoParams for (T0,()) {
    type Target = Params<(U<0, T0>,)>;
    fn into_params() -> Self::Target {
        Params((U::<0, _>(PhantomData),))
    }
}
impl<T0: IntoField, T1: IntoField> IntoParams for (T0, T1) {
    type Target = Params<(U<0, T0>, U<1, T1>)>;
    fn into_params() -> Self::Target {
        Params((U::<0, _>(PhantomData), U::<1, _>(PhantomData)))
    }
}
impl<T0: IntoField, T1: IntoField> IntoParams for (T0, T1, ()) {
    type Target = Params<(U<0, T0>, U<1, T1>)>;
    fn into_params() -> Self::Target {
        Params((U::<0, _>(PhantomData), U::<1, _>(PhantomData)))
    }
}
impl<T0: IntoField, T1: IntoField> IntoParams for (T0, (T1, ())) {
    type Target = Params<(U<0, T0>, U<1, T1>)>;
    fn into_params() -> Self::Target {
        Params((U::<0, _>(PhantomData), U::<1, _>(PhantomData)))
    }
}

impl<T0: IntoField, T1: IntoField, T2: IntoField> IntoParams for (T0, T1, T2) {
    type Target = Params<(U<0, T0>, U<1, T1>, U<2, T2>)>;
    fn into_params() -> Self::Target {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
        ))
    }
}
impl<T0: IntoField, T1: IntoField, T2: IntoField> IntoParams for (T0, T1, T2, ()) {
    type Target = Params<(U<0, T0>, U<1, T1>, U<2, T2>)>;
    fn into_params() -> Self::Target {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
        ))
    }
}

impl<T0: IntoField, T1: IntoField, T2: IntoField> IntoParams for (T0, T1, (T2, ())) {
    type Target = Params<(U<0, T0>, U<1, T1>, U<2, T2>)>;
    fn into_params() -> Self::Target {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
        ))
    }
}

impl<T0: IntoField, T1: IntoField, T2: IntoField> IntoParams for (T0, (T1, T2, ())) {
    type Target = Params<(U<0, T0>, U<1, T1>, U<2, T2>)>;
    fn into_params() -> Self::Target {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
        ))
    }
}

impl<T0: IntoField, T1: IntoField, T2: IntoField, T3: IntoField> IntoParams for (T0, T1, T2, T3) {
    type Target = Params<(U<0, T0>, U<1, T1>, U<2, T2>, U<3, T3>)>;
    fn into_params() -> Self::Target {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
            U::<3, _>(PhantomData),
        ))
    }
}
impl<T0: IntoField, T1: IntoField, T2: IntoField, T3: IntoField> IntoParams for (T0, T1, T2, T3, ()) {
    type Target = Params<(U<0, T0>, U<1, T1>, U<2, T2>, U<3, T3>)>;
    fn into_params() -> Self::Target {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
            U::<3, _>(PhantomData),
        ))
    }
}
impl<T0: IntoField, T1: IntoField, T2: IntoField, T3: IntoField> IntoParams for (T0, T1, T2, (T3, ())) {
    type Target = Params<(U<0, T0>, U<1, T1>, U<2, T2>, U<3, T3>)>;
    fn into_params() -> Self::Target {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
            U::<3, _>(PhantomData),
        ))
    }
}
impl<T0: IntoField, T1: IntoField, T2: IntoField, T3: IntoField> IntoParams for (T0, T1, (T2, T3, ())) {
    type Target = Params<(U<0, T0>, U<1, T1>, U<2, T2>, U<3, T3>)>;
    fn into_params() -> Self::Target {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
            U::<3, _>(PhantomData),
        ))
    }
}
impl<T0: IntoField, T1: IntoField, T2: IntoField, T3: IntoField> IntoParams for (T0, (T1, T2, T3, ())) {
    type Target = Params<(U<0, T0>, U<1, T1>, U<2, T2>, U<3, T3>)>;
    fn into_params() -> Self::Target {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
            U::<3, _>(PhantomData),
        ))
    }
}
impl<T0: IntoField, T1: IntoField, T2: IntoField, T3: IntoField> IntoParams for (T0, T1, (T2, (T3, ()))) {
    type Target = Params<(U<0, T0>, U<1, T1>, U<2, T2>, U<3, T3>)>;
    fn into_params() -> Self::Target {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
            U::<3, _>(PhantomData),
        ))
    }
}
impl<T0: IntoField, T1: IntoField, T2: IntoField, T3: IntoField> IntoParams for (T0, (T1, T2, (T3, ()))) {
    type Target = Params<(U<0, T0>, U<1, T1>, U<2, T2>, U<3, T3>)>;
    fn into_params() -> Self::Target {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
            U::<3, _>(PhantomData),
        ))
    }
}
impl<T0: IntoField, T1: IntoField, T2: IntoField, T3: IntoField> IntoParams for (T0, (T1, (T2, (T3, ())))) {
    type Target = Params<(U<0, T0>, U<1, T1>, U<2, T2>, U<3, T3>)>;
    fn into_params() -> Self::Target {
        Params((
            U::<0, _>(PhantomData),
            U::<1, _>(PhantomData),
            U::<2, _>(PhantomData),
            U::<3, _>(PhantomData),
        ))
    }
}



// impl<T0, T1> IntoParams<2> for (U<0, T0>, U<1, T1>) {
//     type Target = (U<0, T0>, U<1, T1>);
//     fn into_params(self) -> Params<2, (U<0, T0>, U<1, T1>)> {
//         Params::<2, _>((self.0, self.1))
//     }
// }

// impl<T0, T1, P0, P1, P:IntoParams<2, Target = (U<0, P0>, U<1, P1>)>> IntoParams<4> for (U<0, T0>, U<1, T1>, P) {
//     type Target = (U<0, T0>, U<1, T1>, U<2, P0>, U<3, P1>);
//     fn into_params(self) -> Params<4, Self::Target> {
//         let (t0, t1, p) = self;
//         let (p0, p1) = p.into_params().0;
//         Params::<4, _>((
//             t0,
//             t1,
//             <U<0, _> as MoveTo<2>>::move_to(p0),
//             <U<1, _> as MoveTo<3>>::move_to(p1)
//         ))
        
//     }
// }
