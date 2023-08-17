
use xdd::{BDDFactory, DecisionDiagramFactory, NoMultiplicity, VariableIndex, ZDDFactory};


fn test_xdd<F: DecisionDiagramFactory<u32,NoMultiplicity>>() {
    let num_variables = 7;
    let mut factory = F::new(num_variables);
    let v0 = factory.single_variable(VariableIndex(0));
    let v1 = factory.single_variable(VariableIndex(1));
    let v2 = factory.single_variable(VariableIndex(2));
    let v3 = factory.single_variable(VariableIndex(3));
    let v4 = factory.single_variable(VariableIndex(4));
    let v5 = factory.single_variable(VariableIndex(5));
    let v6 = factory.single_variable(VariableIndex(6));
    assert_eq!(Some(vec![VariableIndex(0)]),factory.find_satisfying_solution_with_minimum_number_of_variables(v0));
    assert_eq!(Some(vec![VariableIndex(1)]),factory.find_satisfying_solution_with_minimum_number_of_variables(v1));
    assert_eq!(Some(vec![VariableIndex(2)]),factory.find_satisfying_solution_with_minimum_number_of_variables(v2));
    assert_eq!(Some(vec![VariableIndex(3)]),factory.find_satisfying_solution_with_minimum_number_of_variables(v3));
    assert_eq!(Some(vec![VariableIndex(4)]),factory.find_satisfying_solution_with_minimum_number_of_variables(v4));
    assert_eq!(Some(vec![VariableIndex(5)]),factory.find_satisfying_solution_with_minimum_number_of_variables(v5));
    assert_eq!(Some(vec![VariableIndex(6)]),factory.find_satisfying_solution_with_minimum_number_of_variables(v6));
    let v0_and_v1 = factory.and(v0,v1);
    assert_eq!(Some(vec![VariableIndex(0),VariableIndex(1)]),factory.find_satisfying_solution_with_minimum_number_of_variables(v0_and_v1));
    let v2_and_v3 = factory.and(v2,v3);
    assert_eq!(Some(vec![VariableIndex(2),VariableIndex(3)]),factory.find_satisfying_solution_with_minimum_number_of_variables(v2_and_v3));
    let v0_or_v1 = factory.or(v0,v1);
    assert_eq!(Some(vec![VariableIndex(1)]),factory.find_satisfying_solution_with_minimum_number_of_variables(v0_or_v1));
    let v2_or_v4 = factory.or(v2,v4);
    assert_eq!(Some(vec![VariableIndex(4)]),factory.find_satisfying_solution_with_minimum_number_of_variables(v2_or_v4));
    let v2_or_v4_and_v0_or_v1 = factory.and(v2_or_v4,v0_or_v1);
    assert_eq!(Some(vec![VariableIndex(1),VariableIndex(4)]),factory.find_satisfying_solution_with_minimum_number_of_variables(v2_or_v4_and_v0_or_v1));
    let one_of_2_3_4 = factory.exactly_one_of(&[VariableIndex(2),VariableIndex(3),VariableIndex(4)]);
    assert_eq!(Some(vec![VariableIndex(4)]),factory.find_satisfying_solution_with_minimum_number_of_variables(one_of_2_3_4));
    let v0_or_v2_and_v3 = factory.or(v0,v2_and_v3);
    assert_eq!(Some(vec![VariableIndex(0)]),factory.find_satisfying_solution_with_minimum_number_of_variables(v0_or_v2_and_v3));
}


#[test]
fn test_bdd() {
    test_xdd::<BDDFactory<u32,NoMultiplicity>>()
}

#[test]
fn test_zdd() {
    test_xdd::<ZDDFactory<u32,NoMultiplicity>>()
}
