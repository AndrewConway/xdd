use xdd::{NodeIndex, VariableIndex};
use xdd::xdd_representations::{NodeList, XDDBase};


#[test]
fn zdd_without_lookup() {
    let mut factory = NodeList::default();
    assert_eq!(0, factory.len());

    let v0 = factory.single_variable_zdd(VariableIndex(0),2);
    assert_eq!(2, factory.len());
    assert_eq!(false, factory.evaluate_zdd(v0, &[false, false]));
    assert_eq!(true, factory.evaluate_zdd(v0, &[true, false]));
    assert_eq!(false, factory.evaluate_zdd(v0, &[false, true]));
    assert_eq!(true, factory.evaluate_zdd(v0, &[true, true]));
    let v0_duplicate = factory.single_variable_zdd(VariableIndex(0),2);
    assert_eq!(v0, v0_duplicate);
    assert_eq!(2, factory.len());

    let v1 = factory.single_variable_zdd(VariableIndex(1),2);
    assert_eq!(false, factory.evaluate_zdd(v1, &[false, false]));
    assert_eq!(false, factory.evaluate_zdd(v1, &[true, false]));
    assert_eq!(true, factory.evaluate_zdd(v1, &[false, true]));
    assert_eq!(true, factory.evaluate_zdd(v1, &[true, true]));
    assert_eq!(4, factory.len());
    let v1_duplicate = factory.single_variable_zdd(VariableIndex(1),2);
    assert_eq!(v1, v1_duplicate);
    assert_eq!(4, factory.len());

    let not_v0 = factory.not_zdd(v0,VariableIndex(0),2);
    assert_eq!(6,factory.len());
    assert_eq!(true,factory.evaluate_bdd(not_v0,&[false,false]));
    assert_eq!(false,factory.evaluate_bdd(not_v0,&[true,false]));
    assert_eq!(true,factory.evaluate_bdd(not_v0,&[false,true]));
    assert_eq!(false,factory.evaluate_bdd(not_v0,&[true,true]));

    let not_v0_duplicate = factory.not_zdd(v0,VariableIndex(0),2);
    assert_eq!(not_v0_duplicate,not_v0);
    assert_eq!(6,factory.len());

}

#[test]
fn bdd_without_lookup() {
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

    let not_v0 = factory.not_bdd(v0);
    assert_eq!(4,factory.len());
    assert_eq!(true,factory.evaluate_bdd(not_v0,&[false,false]));
    assert_eq!(false,factory.evaluate_bdd(not_v0,&[true,false]));
    assert_eq!(true,factory.evaluate_bdd(not_v0,&[false,true]));
    assert_eq!(false,factory.evaluate_bdd(not_v0,&[true,true]));

    let not_and_v0_v1 = factory.not_bdd(and_v0_v1);
    assert_eq!(6,factory.len());
    assert_eq!(true,factory.evaluate_bdd(not_and_v0_v1,&[false,false]));
    assert_eq!(true,factory.evaluate_bdd(not_and_v0_v1,&[true,false]));
    assert_eq!(true,factory.evaluate_bdd(not_and_v0_v1,&[false,true]));
    assert_eq!(false,factory.evaluate_bdd(not_and_v0_v1,&[true,true]));

    assert_eq!(NodeIndex::FALSE,factory.and_bdd(not_and_v0_v1,and_v0_v1));
    assert_eq!(NodeIndex::FALSE,factory.and_bdd(v0,not_v0));
    assert_eq!(6,factory.len());
}