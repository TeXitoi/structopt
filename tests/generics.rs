
use structopt::StructOpt;

#[test]
fn generic_struct_flatten() {

    #[derive(StructOpt,PartialEq,Debug)]
    struct Inner{
        pub answer: isize
    }

    #[derive(StructOpt,PartialEq,Debug)]
    struct Outer<T:StructOpt>{
        #[structopt(flatten)]
        pub inner: T
    }

    assert_eq!(
        Outer{inner: Inner{ answer: 42 }},
        Outer::from_iter(&[ "--answer",  "42" ])
    )
}

#[test]
fn generic_struct_flatten_w_where_clause() {

    #[derive(StructOpt,PartialEq,Debug)]
    struct Inner{
        pub answer: isize
    }

    #[derive(StructOpt,PartialEq,Debug)]
    struct Outer<T> where T:StructOpt {
        #[structopt(flatten)]
        pub inner: T
    }

    assert_eq!(
        Outer{inner: Inner{ answer: 42 }},
        Outer::from_iter(&[ "--answer",  "42" ])
    )
}

#[test]
fn generic_enum() {

    #[derive(StructOpt,PartialEq,Debug)]
    struct Inner{
        pub answer: isize
    }

    #[derive(StructOpt,PartialEq,Debug)]
    enum GenericEnum<T: StructOpt> {

        Start(T),
        Stop,
    }

    assert_eq!(
        GenericEnum::Start(Inner{answer: 42}),
        GenericEnum::from_iter(&[ "test", "start", "42" ])
    )

}

#[test]
fn generic_enum_w_where_clause() {

    #[derive(StructOpt,PartialEq,Debug)]
    struct Inner{
        pub answer: isize
    }

    #[derive(StructOpt,PartialEq,Debug)]
    enum GenericEnum<T> where T: StructOpt {

        Start(T),
        Stop,
    }

    assert_eq!(
        GenericEnum::Start(Inner{answer: 42}),
        GenericEnum::from_iter(&[ "test", "start", "42" ])
    )

}

#[test]
fn generic_w_fromstr_trait_bound() {

    use std::str::FromStr;

    #[derive(StructOpt,PartialEq,Debug)]
    struct Opt<T> where T:FromStr
    {
        answer: T
    }

    assert_eq!(
        Opt::<isize>{answer:42},
        Opt::<isize>::from_iter([& "--answer", "42" ])
    )
}
