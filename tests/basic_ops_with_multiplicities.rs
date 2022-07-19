use std::collections::HashMap;
//use std::fs::File;
use xdd::{NodeIndex, NoMultiplicity, VariableIndex};
use xdd::generating_function::{GeneratingFunctionSplitByMultiplicity, SingleVariableGeneratingFunction};
use xdd::xdd_with_multiplicity::{NodeList, NodeListWithFastLookup, XDDBase};


#[test]
fn zdd_without_lookup() { zdd_basic_ops::<NodeList<usize, NoMultiplicity>>() }
#[test]
fn zdd_with_lookup() { zdd_basic_ops::<NodeListWithFastLookup<usize, NoMultiplicity>>() }

fn zdd_basic_ops<F:XDDBase<usize, NoMultiplicity>+Default>() {
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


    let not_v0 = factory.not_zdd(v0,VariableIndex(0),2,&mut HashMap::new());
    // println!("{}",not_v0);
    // not_v0 should be just v1?true:true.
    assert_eq!(4,factory.len());
    assert!(!not_v0.is_sink());
    assert_eq!(VariableIndex(1),factory.node_incorporating_multiplicity(not_v0).variable);
    assert_eq!(NodeIndex::TRUE, factory.node_incorporating_multiplicity(not_v0).hi);
    assert_eq!(NodeIndex::TRUE, factory.node_incorporating_multiplicity(not_v0).lo);
    assert_eq!(true,factory.evaluate_zdd(not_v0,&[false,false]));
    assert_eq!(false,factory.evaluate_zdd(not_v0,&[true,false]));
    assert_eq!(true,factory.evaluate_zdd(not_v0,&[false,true]));
    assert_eq!(false,factory.evaluate_zdd(not_v0,&[true,true]));

    let not_v0_duplicate = factory.not_zdd(v0,VariableIndex(0),2,&mut HashMap::new());
    assert_eq!(not_v0_duplicate,not_v0);
    assert_eq!(4,factory.len());

    let and_v0_v1 = factory.mul_zdd(v0,v1,&mut HashMap::new());
    assert_eq!(5,factory.len());
    assert_eq!(false,factory.evaluate_zdd(and_v0_v1,&[false,false]));
    assert_eq!(false,factory.evaluate_zdd(and_v0_v1,&[true,false]));
    assert_eq!(false,factory.evaluate_zdd(and_v0_v1,&[false,true]));
    assert_eq!(true,factory.evaluate_zdd(and_v0_v1,&[true,true]));
    let and_v1_v0 = factory.mul_zdd(v1,v0,&mut HashMap::new());
    assert_eq!(and_v0_v1,and_v1_v0);
    assert_eq!(5,factory.len());

    let or_v0_v1 = factory.sum_zdd(v0,v1,&mut HashMap::new());
    assert_eq!(6,factory.len());
    assert_eq!(false,factory.evaluate_zdd(or_v0_v1,&[false,false]));
    assert_eq!(true,factory.evaluate_zdd(or_v0_v1,&[true,false]));
    assert_eq!(true,factory.evaluate_zdd(or_v0_v1,&[false,true]));
    assert_eq!(true,factory.evaluate_zdd(or_v0_v1,&[true,true]));
    let or_v1_v0 = factory.sum_zdd(v1,v0,&mut HashMap::new());
    assert_eq!(or_v0_v1,or_v1_v0);
    assert_eq!(6,factory.len());

    // check enumerations
    assert_eq!(2,factory.number_solutions_zdd::<u64>(v1,2));
    assert_eq!(2,factory.number_solutions_zdd::<u64>(v0,2));
    assert_eq!(2,factory.number_solutions_zdd::<u64>(not_v0,2));
    assert_eq!(1,factory.number_solutions_zdd::<u64>(and_v0_v1,2));
    assert_eq!(3,factory.number_solutions_zdd::<u64>(or_v0_v1,2));

    assert_eq!(SingleVariableGeneratingFunction(vec![1]),factory.number_solutions_zdd::<SingleVariableGeneratingFunction::<u64>>(NodeIndex::TRUE, 2));
    assert_eq!(SingleVariableGeneratingFunction(vec![]),factory.number_solutions_zdd::<SingleVariableGeneratingFunction::<u64>>(NodeIndex::FALSE, 2));
    assert_eq!(SingleVariableGeneratingFunction(vec![0,1,1]),factory.number_solutions_zdd::<SingleVariableGeneratingFunction::<u64>>(v1,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![0,1,1]),factory.number_solutions_zdd::<SingleVariableGeneratingFunction::<u64>>(v0,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![1,1]),factory.number_solutions_zdd::<SingleVariableGeneratingFunction::<u64>>(not_v0,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![0,0,1]),factory.number_solutions_zdd::<SingleVariableGeneratingFunction::<u64>>(and_v0_v1,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![0,2,1]),factory.number_solutions_zdd::<SingleVariableGeneratingFunction::<u64>>(or_v0_v1,2));

    // Check GC
    let map = factory.gc([or_v0_v1,and_v0_v1]);
    assert_eq!(4,factory.len());
    let or_v0_v1 = map.rename(or_v0_v1).unwrap();
    let and_v0_v1 = map.rename(and_v0_v1).unwrap();
    //factory.print(or_v0_v1);
    assert_eq!(false,factory.evaluate_zdd(or_v0_v1,&[false,false]));
    assert_eq!(true,factory.evaluate_zdd(or_v0_v1,&[true,false]));
    assert_eq!(true,factory.evaluate_zdd(or_v0_v1,&[false,true]));
    assert_eq!(true,factory.evaluate_zdd(or_v0_v1,&[true,true]));

    assert_eq!(false,factory.evaluate_zdd(and_v0_v1,&[false,false]));
    assert_eq!(false,factory.evaluate_zdd(and_v0_v1,&[true,false]));
    assert_eq!(false,factory.evaluate_zdd(and_v0_v1,&[false,true]));
    assert_eq!(true,factory.evaluate_zdd(and_v0_v1,&[true,true]));

    assert_eq!(or_v0_v1,factory.sum_zdd(or_v0_v1,and_v0_v1,&mut HashMap::new()));

    let xor_v0_v1 = factory.exactly_one_of_zdd(&[VariableIndex(0),VariableIndex(1)],2);
    assert_eq!(6,factory.len());
    assert_eq!(false,factory.evaluate_zdd(xor_v0_v1,&[false,false]));
    assert_eq!(true,factory.evaluate_zdd(xor_v0_v1,&[true,false]));
    assert_eq!(true,factory.evaluate_zdd(xor_v0_v1,&[false,true]));
    assert_eq!(false,factory.evaluate_zdd(xor_v0_v1,&[true,true]));

}


#[test]
fn multiplicities_zdd_without_lookup() { multiplicities_zdd_basic_ops::<NodeList<usize, u32>>() }
#[test]
fn multiplicities_zdd_with_lookup() { multiplicities_zdd_basic_ops::<NodeListWithFastLookup<usize, u32>>() }

fn multiplicities_zdd_basic_ops<F:XDDBase<usize, u32>+Default>() {
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


    let not_v0 = factory.not_zdd(v0,VariableIndex(0),2,&mut HashMap::new());
    // println!("{}",not_v0);
    // not_v0 should be just v1?true:true.
    assert_eq!(4,factory.len());
    assert!(!not_v0.is_sink());
    assert_eq!(VariableIndex(1),factory.node_incorporating_multiplicity(not_v0).variable);
    assert_eq!(NodeIndex::TRUE, factory.node_incorporating_multiplicity(not_v0).hi);
    assert_eq!(NodeIndex::TRUE, factory.node_incorporating_multiplicity(not_v0).lo);
    assert_eq!(true,factory.evaluate_zdd(not_v0,&[false,false]));
    assert_eq!(false,factory.evaluate_zdd(not_v0,&[true,false]));
    assert_eq!(true,factory.evaluate_zdd(not_v0,&[false,true]));
    assert_eq!(false,factory.evaluate_zdd(not_v0,&[true,true]));

    let not_v0_duplicate = factory.not_zdd(v0,VariableIndex(0),2,&mut HashMap::new());
    assert_eq!(not_v0_duplicate,not_v0);
    assert_eq!(4,factory.len());

    let and_v0_v1 = factory.mul_zdd(v0,v1,&mut HashMap::new());
    assert_eq!(5,factory.len());
    assert_eq!(false,factory.evaluate_zdd(and_v0_v1,&[false,false]));
    assert_eq!(false,factory.evaluate_zdd(and_v0_v1,&[true,false]));
    assert_eq!(false,factory.evaluate_zdd(and_v0_v1,&[false,true]));
    assert_eq!(true,factory.evaluate_zdd(and_v0_v1,&[true,true]));
    let and_v1_v0 = factory.mul_zdd(v1,v0,&mut HashMap::new());
    assert_eq!(and_v0_v1,and_v1_v0);
    assert_eq!(5,factory.len());

    let or_v0_v1 = factory.sum_zdd(v0,v1,&mut HashMap::new());
    assert_eq!(7,factory.len());
    assert_eq!(false,factory.evaluate_zdd(or_v0_v1,&[false,false]));
    assert_eq!(true,factory.evaluate_zdd(or_v0_v1,&[true,false]));
    assert_eq!(true,factory.evaluate_zdd(or_v0_v1,&[false,true]));
    assert_eq!(true,factory.evaluate_zdd(or_v0_v1,&[true,true]));
    let or_v1_v0 = factory.sum_zdd(v1,v0,&mut HashMap::new());
    assert_eq!(or_v0_v1,or_v1_v0);
    assert_eq!(7,factory.len());

    // check enumerations
    assert_eq!(2,factory.number_solutions_zdd::<u64>(v1,2));
    assert_eq!(2,factory.number_solutions_zdd::<u64>(v0,2));
    assert_eq!(2,factory.number_solutions_zdd::<u64>(not_v0,2));
    assert_eq!(1,factory.number_solutions_zdd::<u64>(and_v0_v1,2));
    assert_eq!(4,factory.number_solutions_zdd::<u64>(or_v0_v1,2));

    assert_eq!(SingleVariableGeneratingFunction(vec![1]),factory.number_solutions_zdd::<SingleVariableGeneratingFunction::<u64>>(NodeIndex::TRUE, 2));
    assert_eq!(SingleVariableGeneratingFunction(vec![]),factory.number_solutions_zdd::<SingleVariableGeneratingFunction::<u64>>(NodeIndex::FALSE, 2));
    assert_eq!(SingleVariableGeneratingFunction(vec![0,1,1]),factory.number_solutions_zdd::<SingleVariableGeneratingFunction::<u64>>(v1,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![0,1,1]),factory.number_solutions_zdd::<SingleVariableGeneratingFunction::<u64>>(v0,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![1,1]),factory.number_solutions_zdd::<SingleVariableGeneratingFunction::<u64>>(not_v0,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![0,0,1]),factory.number_solutions_zdd::<SingleVariableGeneratingFunction::<u64>>(and_v0_v1,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![0,2,2]),factory.number_solutions_zdd::<SingleVariableGeneratingFunction::<u64>>(or_v0_v1,2));
    let doubled_or = factory.sum_zdd(or_v0_v1,or_v0_v1,&mut HashMap::new());
    let squared_or = factory.mul_zdd(or_v0_v1,or_v0_v1,&mut HashMap::new());
    // factory.make_dot_file(&mut File::create("doubled_or.gv").unwrap(),"x",&[(doubled_or,Some("doubled_or".to_string())),(squared_or,Some("squared_or".to_string())),(or_v0_v1,Some("Or".to_string()))],|v|if v.0==0 {"x".to_string()} else {"y".to_string()}).unwrap();
    assert_eq!(SingleVariableGeneratingFunction(vec![0,4,4]),factory.number_solutions_zdd::<SingleVariableGeneratingFunction::<u64>>(doubled_or,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![0,2,4]),factory.number_solutions_zdd::<SingleVariableGeneratingFunction::<u64>>(squared_or,2));
    assert_eq!(GeneratingFunctionSplitByMultiplicity(vec![2,1]),factory.number_solutions_zdd::<GeneratingFunctionSplitByMultiplicity::<u64>>(or_v0_v1,2));
    assert_eq!(GeneratingFunctionSplitByMultiplicity(vec![0,2,0,1]),factory.number_solutions_zdd::<GeneratingFunctionSplitByMultiplicity::<u64>>(doubled_or,2));
    assert_eq!(GeneratingFunctionSplitByMultiplicity(vec![2,0,0,1]),factory.number_solutions_zdd::<GeneratingFunctionSplitByMultiplicity::<u64>>(squared_or,2));

    // Check GC
    let map = factory.gc([or_v0_v1,and_v0_v1]);
    assert_eq!(4,factory.len());
    let or_v0_v1 = map.rename(or_v0_v1).unwrap();
    let and_v0_v1 = map.rename(and_v0_v1).unwrap();
    //factory.print(or_v0_v1);
    assert_eq!(false,factory.evaluate_zdd(or_v0_v1,&[false,false]));
    assert_eq!(true,factory.evaluate_zdd(or_v0_v1,&[true,false]));
    assert_eq!(true,factory.evaluate_zdd(or_v0_v1,&[false,true]));
    assert_eq!(true,factory.evaluate_zdd(or_v0_v1,&[true,true]));

    assert_eq!(false,factory.evaluate_zdd(and_v0_v1,&[false,false]));
    assert_eq!(false,factory.evaluate_zdd(and_v0_v1,&[true,false]));
    assert_eq!(false,factory.evaluate_zdd(and_v0_v1,&[false,true]));
    assert_eq!(true,factory.evaluate_zdd(and_v0_v1,&[true,true]));

    assert_ne!(or_v0_v1,factory.sum_zdd(or_v0_v1,and_v0_v1,&mut HashMap::new()));

    let xor_v0_v1 = factory.exactly_one_of_zdd(&[VariableIndex(0),VariableIndex(1)],2);
    assert_eq!(8,factory.len());
    assert_eq!(false,factory.evaluate_zdd(xor_v0_v1,&[false,false]));
    assert_eq!(true,factory.evaluate_zdd(xor_v0_v1,&[true,false]));
    assert_eq!(true,factory.evaluate_zdd(xor_v0_v1,&[false,true]));
    assert_eq!(false,factory.evaluate_zdd(xor_v0_v1,&[true,true]));

}

#[test]
fn bdd_without_lookup() { bdd_basic_ops::<NodeList<usize, NoMultiplicity>>() }
#[test]
fn bdd_with_lookup() { bdd_basic_ops::<NodeListWithFastLookup<usize, NoMultiplicity>>() }

fn bdd_basic_ops<F:XDDBase<usize, NoMultiplicity>+Default>() {
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

    let and_v0_v1 = factory.mul_bdd(v0,v1,&mut HashMap::new());
    assert_eq!(3,factory.len());
    assert_eq!(false,factory.evaluate_bdd(and_v0_v1,&[false,false]));
    assert_eq!(false,factory.evaluate_bdd(and_v0_v1,&[true,false]));
    assert_eq!(false,factory.evaluate_bdd(and_v0_v1,&[false,true]));
    assert_eq!(true,factory.evaluate_bdd(and_v0_v1,&[true,true]));
    let and_v1_v0 = factory.mul_bdd(v1,v0,&mut HashMap::new());
    assert_eq!(and_v0_v1,and_v1_v0);
    assert_eq!(3,factory.len());

    let not_v0 = factory.not_bdd(v0,&mut HashMap::new());
    assert_eq!(4,factory.len());
    assert_eq!(true,factory.evaluate_bdd(not_v0,&[false,false]));
    assert_eq!(false,factory.evaluate_bdd(not_v0,&[true,false]));
    assert_eq!(true,factory.evaluate_bdd(not_v0,&[false,true]));
    assert_eq!(false,factory.evaluate_bdd(not_v0,&[true,true]));

    let not_and_v0_v1 = factory.not_bdd(and_v0_v1,&mut HashMap::new());
    assert_eq!(6,factory.len());
    assert_eq!(true,factory.evaluate_bdd(not_and_v0_v1,&[false,false]));
    assert_eq!(true,factory.evaluate_bdd(not_and_v0_v1,&[true,false]));
    assert_eq!(true,factory.evaluate_bdd(not_and_v0_v1,&[false,true]));
    assert_eq!(false,factory.evaluate_bdd(not_and_v0_v1,&[true,true]));

    assert_eq!(NodeIndex::FALSE, factory.mul_bdd(not_and_v0_v1, and_v0_v1, &mut HashMap::new()));
    assert_eq!(NodeIndex::FALSE, factory.mul_bdd(v0, not_v0, &mut HashMap::new()));
    assert_eq!(6,factory.len());

    let or_v0_v1 = factory.sum_bdd(v0,v1,&mut HashMap::new());
    assert_eq!(7,factory.len());
    assert_eq!(false,factory.evaluate_bdd(or_v0_v1,&[false,false]));
    assert_eq!(true,factory.evaluate_bdd(or_v0_v1,&[true,false]));
    assert_eq!(true,factory.evaluate_bdd(or_v0_v1,&[false,true]));
    assert_eq!(true,factory.evaluate_bdd(or_v0_v1,&[true,true]));
    let or_v1_v0 = factory.sum_bdd(v1,v0,&mut HashMap::new());
    assert_eq!(or_v0_v1,or_v1_v0);
    assert_eq!(7,factory.len());

    // check enumerations
    assert_eq!(2,factory.number_solutions_bdd::<u64>(v1,2));
    assert_eq!(2,factory.number_solutions_bdd::<u64>(v0,2));
    assert_eq!(2,factory.number_solutions_bdd::<u64>(not_v0,2));
    assert_eq!(1,factory.number_solutions_bdd::<u64>(and_v0_v1,2));
    assert_eq!(3,factory.number_solutions_bdd::<u64>(or_v0_v1,2));
    assert_eq!(3,factory.number_solutions_bdd::<u64>(not_and_v0_v1,2));
    //assert_eq!(3,factory.number_solutions_bdd::<u64>(or_v0_v1,2));

    assert_eq!(SingleVariableGeneratingFunction(vec![1,2,1]),factory.number_solutions_bdd::<SingleVariableGeneratingFunction::<u64>>(NodeIndex::TRUE, 2));
    assert_eq!(SingleVariableGeneratingFunction(vec![]),factory.number_solutions_bdd::<SingleVariableGeneratingFunction::<u64>>(NodeIndex::FALSE, 2));
    assert_eq!(SingleVariableGeneratingFunction(vec![0,1,1]),factory.number_solutions_bdd::<SingleVariableGeneratingFunction::<u64>>(v1,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![0,1,1]),factory.number_solutions_bdd::<SingleVariableGeneratingFunction::<u64>>(v0,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![1,1]),factory.number_solutions_bdd::<SingleVariableGeneratingFunction::<u64>>(not_v0,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![0,0,1]),factory.number_solutions_bdd::<SingleVariableGeneratingFunction::<u64>>(and_v0_v1,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![0,2,1]),factory.number_solutions_bdd::<SingleVariableGeneratingFunction::<u64>>(or_v0_v1,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![1,2]),factory.number_solutions_bdd::<SingleVariableGeneratingFunction::<u64>>(not_and_v0_v1,2));
    let doubled_or = factory.sum_bdd(or_v0_v1,or_v0_v1,&mut HashMap::new());
    let squared_or = factory.mul_bdd(or_v0_v1,or_v0_v1,&mut HashMap::new());
    // factory.make_dot_file(&mut File::create("doubled_or.gv").unwrap(),"x",&[(doubled_or,Some("doubled_or".to_string())),(squared_or,Some("squared_or".to_string())),(or_v0_v1,Some("Or".to_string()))],|v|if v.0==0 {"x".to_string()} else {"y".to_string()}).unwrap();
    assert_eq!(SingleVariableGeneratingFunction(vec![0,2,1]),factory.number_solutions_bdd::<SingleVariableGeneratingFunction::<u64>>(doubled_or,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![0,2,1]),factory.number_solutions_bdd::<SingleVariableGeneratingFunction::<u64>>(squared_or,2));

    // Check GC
    let map = factory.gc([or_v0_v1,and_v0_v1]);
    assert_eq!(3,factory.len());
    let or_v0_v1 = map.rename(or_v0_v1).unwrap();
    let and_v0_v1 = map.rename(and_v0_v1).unwrap();
    //factory.print(or_v0_v1);
    assert_eq!(false,factory.evaluate_bdd(or_v0_v1,&[false,false]));
    assert_eq!(true,factory.evaluate_bdd(or_v0_v1,&[true,false]));
    assert_eq!(true,factory.evaluate_bdd(or_v0_v1,&[false,true]));
    assert_eq!(true,factory.evaluate_bdd(or_v0_v1,&[true,true]));

    assert_eq!(false,factory.evaluate_bdd(and_v0_v1,&[false,false]));
    assert_eq!(false,factory.evaluate_bdd(and_v0_v1,&[true,false]));
    assert_eq!(false,factory.evaluate_bdd(and_v0_v1,&[false,true]));
    assert_eq!(true,factory.evaluate_bdd(and_v0_v1,&[true,true]));

    assert_eq!(or_v0_v1,factory.sum_bdd(or_v0_v1,and_v0_v1,&mut HashMap::new()));

    let xor_v0_v1 = factory.exactly_one_of_bdd(&[VariableIndex(0),VariableIndex(1)]);
    assert_eq!(5,factory.len());
    assert_eq!(false,factory.evaluate_bdd(xor_v0_v1,&[false,false]));
    assert_eq!(true,factory.evaluate_bdd(xor_v0_v1,&[true,false]));
    assert_eq!(true,factory.evaluate_bdd(xor_v0_v1,&[false,true]));
    assert_eq!(false,factory.evaluate_bdd(xor_v0_v1,&[true,true]));

}

#[test]
fn multiplicities_bdd_without_lookup() { bdd_basic_ops_with_multiplicities::<NodeList<usize, u32>>() }
#[test]
fn multiplicities_bdd_with_lookup() { bdd_basic_ops_with_multiplicities::<NodeListWithFastLookup<usize, u32>>() }


fn bdd_basic_ops_with_multiplicities<F:XDDBase<usize, u32>+Default>() {
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

    let and_v0_v1 = factory.mul_bdd(v0,v1,&mut HashMap::new());
    assert_eq!(3,factory.len());
    assert_eq!(false,factory.evaluate_bdd(and_v0_v1,&[false,false]));
    assert_eq!(false,factory.evaluate_bdd(and_v0_v1,&[true,false]));
    assert_eq!(false,factory.evaluate_bdd(and_v0_v1,&[false,true]));
    assert_eq!(true,factory.evaluate_bdd(and_v0_v1,&[true,true]));
    let and_v1_v0 = factory.mul_bdd(v1,v0,&mut HashMap::new());
    assert_eq!(and_v0_v1,and_v1_v0);
    assert_eq!(3,factory.len());

    let not_v0 = factory.not_bdd(v0,&mut HashMap::new());
    assert_eq!(4,factory.len());
    assert_eq!(true,factory.evaluate_bdd(not_v0,&[false,false]));
    assert_eq!(false,factory.evaluate_bdd(not_v0,&[true,false]));
    assert_eq!(true,factory.evaluate_bdd(not_v0,&[false,true]));
    assert_eq!(false,factory.evaluate_bdd(not_v0,&[true,true]));

    let not_and_v0_v1 = factory.not_bdd(and_v0_v1,&mut HashMap::new());
    assert_eq!(6,factory.len());
    assert_eq!(true,factory.evaluate_bdd(not_and_v0_v1,&[false,false]));
    assert_eq!(true,factory.evaluate_bdd(not_and_v0_v1,&[true,false]));
    assert_eq!(true,factory.evaluate_bdd(not_and_v0_v1,&[false,true]));
    assert_eq!(false,factory.evaluate_bdd(not_and_v0_v1,&[true,true]));

    assert_eq!(NodeIndex::FALSE, factory.mul_bdd(not_and_v0_v1, and_v0_v1, &mut HashMap::new()));
    assert_eq!(NodeIndex::FALSE, factory.mul_bdd(v0, not_v0, &mut HashMap::new()));
    assert_eq!(6,factory.len());

    let or_v0_v1 = factory.sum_bdd(v0,v1,&mut HashMap::new());
    assert_eq!(8,factory.len());
    assert_eq!(false,factory.evaluate_bdd(or_v0_v1,&[false,false]));
    assert_eq!(true,factory.evaluate_bdd(or_v0_v1,&[true,false]));
    assert_eq!(true,factory.evaluate_bdd(or_v0_v1,&[false,true]));
    assert_eq!(true,factory.evaluate_bdd(or_v0_v1,&[true,true]));
    let or_v1_v0 = factory.sum_bdd(v1,v0,&mut HashMap::new());
    assert_eq!(or_v0_v1,or_v1_v0);
    assert_eq!(8,factory.len());

    // check enumerations
    assert_eq!(2,factory.number_solutions_bdd::<u64>(v1,2));
    assert_eq!(2,factory.number_solutions_bdd::<u64>(v0,2));
    assert_eq!(2,factory.number_solutions_bdd::<u64>(not_v0,2));
    assert_eq!(1,factory.number_solutions_bdd::<u64>(and_v0_v1,2));
    assert_eq!(4,factory.number_solutions_bdd::<u64>(or_v0_v1,2)); // bigger than without multiplicities!
    assert_eq!(3,factory.number_solutions_bdd::<u64>(not_and_v0_v1,2));
    //assert_eq!(3,factory.number_solutions_bdd::<u64>(or_v0_v1,2));

    assert_eq!(SingleVariableGeneratingFunction(vec![1,2,1]),factory.number_solutions_bdd::<SingleVariableGeneratingFunction::<u64>>(NodeIndex::TRUE, 2));
    assert_eq!(SingleVariableGeneratingFunction(vec![]),factory.number_solutions_bdd::<SingleVariableGeneratingFunction::<u64>>(NodeIndex::FALSE, 2));
    assert_eq!(SingleVariableGeneratingFunction(vec![0,1,1]),factory.number_solutions_bdd::<SingleVariableGeneratingFunction::<u64>>(v1,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![0,1,1]),factory.number_solutions_bdd::<SingleVariableGeneratingFunction::<u64>>(v0,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![1,1]),factory.number_solutions_bdd::<SingleVariableGeneratingFunction::<u64>>(not_v0,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![0,0,1]),factory.number_solutions_bdd::<SingleVariableGeneratingFunction::<u64>>(and_v0_v1,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![0,2,2]),factory.number_solutions_bdd::<SingleVariableGeneratingFunction::<u64>>(or_v0_v1,2));
    let doubled_or = factory.sum_bdd(or_v0_v1,or_v0_v1,&mut HashMap::new());
    let squared_or = factory.mul_bdd(or_v0_v1,or_v0_v1,&mut HashMap::new());
    // factory.make_dot_file(&mut File::create("doubled_or.gv").unwrap(),"x",&[(doubled_or,Some("doubled_or".to_string())),(squared_or,Some("squared_or".to_string())),(or_v0_v1,Some("Or".to_string()))],|v|if v.0==0 {"x".to_string()} else {"y".to_string()}).unwrap();
    assert_eq!(GeneratingFunctionSplitByMultiplicity(vec![2,1]),factory.number_solutions_bdd::<GeneratingFunctionSplitByMultiplicity::<u64>>(or_v0_v1,2));
    assert_eq!(GeneratingFunctionSplitByMultiplicity(vec![0,2,0,1]),factory.number_solutions_bdd::<GeneratingFunctionSplitByMultiplicity::<u64>>(doubled_or,2));
    assert_eq!(GeneratingFunctionSplitByMultiplicity(vec![2,0,0,1]),factory.number_solutions_bdd::<GeneratingFunctionSplitByMultiplicity::<u64>>(squared_or,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![0,4,4]),factory.number_solutions_bdd::<SingleVariableGeneratingFunction::<u64>>(doubled_or,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![0,2,4]),factory.number_solutions_bdd::<SingleVariableGeneratingFunction::<u64>>(squared_or,2));
    assert_eq!(SingleVariableGeneratingFunction(vec![1,2]),factory.number_solutions_bdd::<SingleVariableGeneratingFunction::<u64>>(not_and_v0_v1,2));

    // Check GC
    let map = factory.gc([or_v0_v1,and_v0_v1]);
    assert_eq!(4,factory.len());
    let or_v0_v1 = map.rename(or_v0_v1).unwrap();
    let and_v0_v1 = map.rename(and_v0_v1).unwrap();
    //factory.print(or_v0_v1);
    assert_eq!(false,factory.evaluate_bdd(or_v0_v1,&[false,false]));
    assert_eq!(true,factory.evaluate_bdd(or_v0_v1,&[true,false]));
    assert_eq!(true,factory.evaluate_bdd(or_v0_v1,&[false,true]));
    assert_eq!(true,factory.evaluate_bdd(or_v0_v1,&[true,true]));

    assert_eq!(false,factory.evaluate_bdd(and_v0_v1,&[false,false]));
    assert_eq!(false,factory.evaluate_bdd(and_v0_v1,&[true,false]));
    assert_eq!(false,factory.evaluate_bdd(and_v0_v1,&[false,true]));
    assert_eq!(true,factory.evaluate_bdd(and_v0_v1,&[true,true]));

    assert_ne!(or_v0_v1,factory.sum_bdd(or_v0_v1,and_v0_v1,&mut HashMap::new()));

    let xor_v0_v1 = factory.exactly_one_of_bdd(&[VariableIndex(0),VariableIndex(1)]);
    assert_eq!(8,factory.len());
    assert_eq!(false,factory.evaluate_bdd(xor_v0_v1,&[false,false]));
    assert_eq!(true,factory.evaluate_bdd(xor_v0_v1,&[true,false]));
    assert_eq!(true,factory.evaluate_bdd(xor_v0_v1,&[false,true]));
    assert_eq!(false,factory.evaluate_bdd(xor_v0_v1,&[true,true]));

}
