use xdd::{NodeIndex, VariableIndex};
use xdd::generating_function::SingleVariableGeneratingFunction;
use xdd::xdd_representations::{NodeList, NodeListWithFastLookup, XDDBase};


#[test]
fn zdd_without_lookup() { zdd_basic_ops::<NodeList>() }
#[test]
fn zdd_with_lookup() { zdd_basic_ops::<NodeListWithFastLookup>() }

fn zdd_basic_ops<F:XDDBase+Default>() {
    let mut factory = F::default();
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
    // println!("{}",not_v0);
    // not_v0 should be just v1?true:true.
    assert_eq!(4,factory.len());
    assert!(!not_v0.is_sink());
    assert_eq!(VariableIndex(1),factory.node(not_v0).variable);
    assert_eq!(NodeIndex::TRUE,factory.node(not_v0).hi);
    assert_eq!(NodeIndex::TRUE,factory.node(not_v0).lo);
    assert_eq!(true,factory.evaluate_zdd(not_v0,&[false,false]));
    assert_eq!(false,factory.evaluate_zdd(not_v0,&[true,false]));
    assert_eq!(true,factory.evaluate_zdd(not_v0,&[false,true]));
    assert_eq!(false,factory.evaluate_zdd(not_v0,&[true,true]));

    let not_v0_duplicate = factory.not_zdd(v0,VariableIndex(0),2);
    assert_eq!(not_v0_duplicate,not_v0);
    assert_eq!(4,factory.len());

    let and_v0_v1 = factory.and_zdd(v0,v1);
    assert_eq!(5,factory.len());
    assert_eq!(false,factory.evaluate_zdd(and_v0_v1,&[false,false]));
    assert_eq!(false,factory.evaluate_zdd(and_v0_v1,&[true,false]));
    assert_eq!(false,factory.evaluate_zdd(and_v0_v1,&[false,true]));
    assert_eq!(true,factory.evaluate_zdd(and_v0_v1,&[true,true]));
    let and_v1_v0 = factory.and_zdd(v1,v0);
    assert_eq!(and_v0_v1,and_v1_v0);
    assert_eq!(5,factory.len());

    let or_v0_v1 = factory.or_zdd(v0,v1);
    assert_eq!(6,factory.len());
    assert_eq!(false,factory.evaluate_zdd(or_v0_v1,&[false,false]));
    assert_eq!(true,factory.evaluate_zdd(or_v0_v1,&[true,false]));
    assert_eq!(true,factory.evaluate_zdd(or_v0_v1,&[false,true]));
    assert_eq!(true,factory.evaluate_zdd(or_v0_v1,&[true,true]));
    let or_v1_v0 = factory.or_zdd(v1,v0);
    assert_eq!(or_v0_v1,or_v1_v0);
    assert_eq!(6,factory.len());

    // check enumerations
    assert_eq!(2,factory.number_solutions_zdd::<u64>(v1,2));
    assert_eq!(2,factory.number_solutions_zdd::<u64>(v0,2));
    assert_eq!(2,factory.number_solutions_zdd::<u64>(not_v0,2));
    assert_eq!(1,factory.number_solutions_zdd::<u64>(and_v0_v1,2));
    assert_eq!(3,factory.number_solutions_zdd::<u64>(or_v0_v1,2));

    assert_eq!(SingleVariableGeneratingFunction(vec![1]),factory.number_solutions_zdd::<SingleVariableGeneratingFunction>(NodeIndex::TRUE,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![]),factory.number_solutions_zdd::<SingleVariableGeneratingFunction>(NodeIndex::FALSE,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![0,1,1]),factory.number_solutions_zdd::<SingleVariableGeneratingFunction>(v1,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![0,1,1]),factory.number_solutions_zdd::<SingleVariableGeneratingFunction>(v0,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![1,1]),factory.number_solutions_zdd::<SingleVariableGeneratingFunction>(not_v0,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![0,0,1]),factory.number_solutions_zdd::<SingleVariableGeneratingFunction>(and_v0_v1,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![0,2,1]),factory.number_solutions_zdd::<SingleVariableGeneratingFunction>(or_v0_v1,2));

}

#[test]
fn bdd_without_lookup() { bdd_basic_ops::<NodeList>() }
#[test]
fn bdd_with_lookup() { bdd_basic_ops::<NodeListWithFastLookup>() }

fn bdd_basic_ops<F:XDDBase+Default>() {
    let mut factory = F::default();
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

    // check enumerations
    assert_eq!(2,factory.number_solutions_bdd::<u64>(v1,2));
    assert_eq!(2,factory.number_solutions_bdd::<u64>(v0,2));
    assert_eq!(2,factory.number_solutions_bdd::<u64>(not_v0,2));
    assert_eq!(1,factory.number_solutions_bdd::<u64>(and_v0_v1,2));
    assert_eq!(3,factory.number_solutions_bdd::<u64>(not_and_v0_v1,2));
    //assert_eq!(3,factory.number_solutions_bdd::<u64>(or_v0_v1,2));

    assert_eq!(SingleVariableGeneratingFunction(vec![1,2,1]),factory.number_solutions_bdd::<SingleVariableGeneratingFunction>(NodeIndex::TRUE,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![]),factory.number_solutions_bdd::<SingleVariableGeneratingFunction>(NodeIndex::FALSE,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![0,1,1]),factory.number_solutions_bdd::<SingleVariableGeneratingFunction>(v1,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![0,1,1]),factory.number_solutions_bdd::<SingleVariableGeneratingFunction>(v0,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![1,1]),factory.number_solutions_bdd::<SingleVariableGeneratingFunction>(not_v0,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![0,0,1]),factory.number_solutions_bdd::<SingleVariableGeneratingFunction>(and_v0_v1,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![1,2]),factory.number_solutions_bdd::<SingleVariableGeneratingFunction>(not_and_v0_v1,2));
    //assert_eq!(SingleVariableGeneratingFunction(vec![0,2,1]),factory.number_solutions_bdd::<SingleVariableGeneratingFunction>(or_v0_v1,2));

}

