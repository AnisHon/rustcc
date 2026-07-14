use rcc::{ConstantEvaluator, EvaluationFailure, ExternalDeclaration, Initializer, compile};

fn initializer(source: &str) -> rcc::Expression {
    let compilation = compile(source).unwrap();
    compilation
        .ast
        .declarations
        .iter()
        .find_map(|declaration| {
            let ExternalDeclaration::Declaration(declaration) = declaration else {
                return None;
            };
            let Some(Initializer::Expression(expression)) = &declaration.initializer else {
                return None;
            };
            Some(expression.clone())
        })
        .unwrap()
}

#[test]
fn standalone_evaluator_consumes_typed_ast() {
    let expression = initializer("int value = (2 + 3) * 4;");
    let target = rcc::TargetInfo::default();
    let evaluator = ConstantEvaluator::new(&target, &|_| None);
    assert_eq!(evaluator.evaluate_integer(&expression), Ok(20));
}

#[test]
fn evaluator_distinguishes_runtime_values_and_undefined_behavior() {
    let runtime = initializer("extern int input; int value = input;");
    let target = rcc::TargetInfo::default();
    let evaluator = ConstantEvaluator::new(&target, &|_| None);
    assert!(matches!(
        evaluator.evaluate_integer(&runtime),
        Err(EvaluationFailure::DependsOnRuntimeValue(name)) if name == "input"
    ));

    let division = initializer("int value = 1 / 0;");
    assert!(matches!(
        evaluator.evaluate_integer(&division),
        Err(EvaluationFailure::UndefinedBehavior(_))
    ));
}
