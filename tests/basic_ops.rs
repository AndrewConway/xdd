use xdd::VariableIndex;
use xdd::xdd_representations::{NodeList, XDDBase};

#[test]
fn without_lookup() {
    let mut factory = NodeList::default();
    assert_eq!(0,factory.len());
    let v0 = factory.single_variable(VariableIndex(0));
    assert_eq!(1,factory.len());
    assert_eq!(false,factory.evaluate_bdd(v0,&[false,false]));
    assert_eq!(true,factory.evaluate_bdd(v0,&[true,false]));
    assert_eq!(false,factory.evaluate_bdd(v0,&[false,true]));
    assert_eq!(true,factory.evaluate_bdd(v0,&[true,true]));
    let v0_duplicate = factory.single_variable(VariableIndex(0));
    assert_eq!(v0,v0_duplicate);
    assert_eq!(1,factory.len());
    let v1 = factory.single_variable(VariableIndex(1));
    assert_eq!(false,factory.evaluate_bdd(v1,&[false,false]));
    assert_eq!(false,factory.evaluate_bdd(v1,&[true,false]));
    assert_eq!(true,factory.evaluate_bdd(v1,&[false,true]));
    assert_eq!(true,factory.evaluate_bdd(v1,&[true,true]));
    assert_eq!(2,factory.len());
    let v1_duplicate = factory.single_variable(VariableIndex(1));
    assert_eq!(v1,v1_duplicate);
    assert_eq!(2,factory.len());
    let and_v0_v1 = factory.and_bdd(v0,v1);
    assert_eq!(3,factory.len());
    assert_eq!(false,factory.evaluate_bdd(and_v0_v1,&[false,false]));
    assert_eq!(false,factory.evaluate_bdd(and_v0_v1,&[true,false]));
    assert_eq!(false,factory.evaluate_bdd(and_v0_v1,&[false,true]));
    assert_eq!(true,factory.evaluate_bdd(and_v0_v1,&[true,true]));
    let and_v1_v0 = factory.and_bdd(v1,v0);
    assert_eq!(and_v0_v1,and_v1_v0);
    assert_eq!(3,factory.len());


}