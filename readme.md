Proving the Fibonacci Sequence with [Sonobe](https://github.com/privacy-scaling-explorations/sonobe)'s Nova scheme.

```
    fn step_native(
        &self,
        _i: usize,
        z_i: Vec<F>,
        _external_inputs: Vec<F>,
    ) -> Result<Vec<F>, Error> {
        let new_fib = z_i[0] + z_i[1];
        println!("Current Fibonacci numbers: {}, {}", z_i[1], new_fib);
        Ok(vec![z_i[1], new_fib])
    }

    fn generate_step_constraints(
        &self,
        cs: ConstraintSystemRef<F>,
        _i: usize,
        z_i: Vec<FpVar<F>>,
        _external_inputs: Vec<FpVar<F>>,
    ) -> Result<Vec<FpVar<F>>, SynthesisError> {
        let new_fib = &z_i[0] + &z_i[1];
        Ok(vec![z_i[1].clone(), new_fib])
    }
```