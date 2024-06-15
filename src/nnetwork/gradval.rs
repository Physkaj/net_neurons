use std::{
    cell::RefCell,
    fmt::Display,
    ops::{Add, Div, Mul, Neg, Sub},
    rc::Rc,
};

type Ancestor = Rc<RefCell<Gv>>;

#[derive(Clone, Debug, Default, PartialEq)]
enum GradValOp {
    #[default]
    Noop,
    Neg(Ancestor),
    Exp(Ancestor),
    Log(Ancestor),
    Pow(Ancestor, Ancestor),
    Add(Ancestor, Ancestor),
    Sub(Ancestor, Ancestor),
    Mul(Ancestor, Ancestor),
    Div(Ancestor, Ancestor),
}
impl GradValOp {
    fn op_symb(&self) -> &str {
        match self {
            GradValOp::Noop => "NOOP",
            GradValOp::Neg(_) => "-",
            GradValOp::Exp(_) => "exp",
            GradValOp::Log(_) => "log",
            GradValOp::Pow(_, _) => "^",
            GradValOp::Add(_, _) => "+",
            GradValOp::Sub(_, _) => "-",
            GradValOp::Mul(_, _) => "*",
            GradValOp::Div(_, _) => "/",
        }
    }
}

impl Display for GradValOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GradValOp::Noop => write!(f, "{}", self.op_symb()),
            GradValOp::Neg(a) | GradValOp::Exp(a) | GradValOp::Log(a) => {
                write!(f, "{}({:e})", self.op_symb(), a.borrow()._val)
            }
            GradValOp::Pow(a, b)
            | GradValOp::Add(a, b)
            | GradValOp::Sub(a, b)
            | GradValOp::Mul(a, b)
            | GradValOp::Div(a, b) => write!(
                f,
                "{:e} {} {:e}",
                a.borrow()._val,
                self.op_symb(),
                b.borrow()._val
            ),
        }
    }
}

#[derive(Debug, Default)]
struct Gv {
    _val: f32,
    _grad: Option<f32>, // Partial derivative of root value having called backward() wrt. this value
    _op: GradValOp,     // Operation which the value originated from
}

// Constructors
impl Gv {
    fn from_op(v: f32, op: GradValOp) -> Self {
        Gv {
            _val: v,
            _op: op,
            ..Self::default()
        }
    }
}
impl From<f32> for Gv {
    fn from(value: f32) -> Self {
        Gv {
            _val: value,
            ..Self::default()
        }
    }
}

impl PartialEq for Gv {
    fn eq(&self, other: &Self) -> bool {
        self._val == other._val
    }
}

impl PartialOrd for Gv {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self._val.partial_cmp(&other._val)
    }
}

// Back propagation
impl Gv {
    fn reset_grad_recursively(&mut self) {
        self._grad = None;
        match &self._op {
            GradValOp::Noop => {}
            GradValOp::Neg(a) | GradValOp::Exp(a) | GradValOp::Log(a) => {
                a.borrow_mut().reset_grad_recursively()
            }
            GradValOp::Pow(a, b)
            | GradValOp::Add(a, b)
            | GradValOp::Sub(a, b)
            | GradValOp::Mul(a, b)
            | GradValOp::Div(a, b) => {
                a.borrow_mut().reset_grad_recursively();
                b.borrow_mut().reset_grad_recursively();
            }
        }
    }

    fn calc_grad_recursively(&mut self, grad: f32) {
        self._grad = Some(
            match self._grad {
                Some(g) => g,
                None => 0.,
            } + grad,
        );
        // Calc grad for children
        match &self._op {
            GradValOp::Noop => {}
            GradValOp::Neg(a) => {
                a.borrow_mut().calc_grad_recursively(-grad);
            }
            GradValOp::Exp(a) => {
                let g = a.borrow()._val.exp();
                a.borrow_mut().calc_grad_recursively(g * grad);
            }
            GradValOp::Log(a) => {
                let g = 1. / a.borrow()._val;
                a.borrow_mut().calc_grad_recursively(g * grad);
            }
            GradValOp::Pow(a, b) => {
                let a_val = a.borrow()._val;
                let b_val = b.borrow()._val;
                let g = b_val * a_val.powf(b_val - 1.);
                println!("{} {}", g, grad);
                a.borrow_mut().calc_grad_recursively(g * grad);
                let g = a_val.ln() * a_val.powf(b_val);
                b.borrow_mut().calc_grad_recursively(g * grad);
            }
            GradValOp::Add(a, b) => {
                a.borrow_mut().calc_grad_recursively(grad);
                b.borrow_mut().calc_grad_recursively(grad);
            }
            GradValOp::Sub(a, b) => {
                a.borrow_mut().calc_grad_recursively(grad);
                b.borrow_mut().calc_grad_recursively(-grad);
            }
            GradValOp::Mul(a, b) => {
                a.borrow_mut().calc_grad_recursively(grad * b.borrow()._val);
                b.borrow_mut().calc_grad_recursively(grad * a.borrow()._val);
            }
            GradValOp::Div(a, b) => {
                let a_val = a.borrow()._val;
                let b_val = b.borrow()._val;
                a.borrow_mut().calc_grad_recursively(grad / b_val);
                let g = -a_val / (b_val.powi(2));
                b.borrow_mut().calc_grad_recursively(g * grad);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct GradVal {
    _gv: Rc<RefCell<Gv>>,
}

impl Display for GradVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[GVal: ")?;
        if self._gv.borrow()._op != GradValOp::Noop {
            write!(f, " {} = ", self._gv.borrow()._op)?;
        }
        write!(f, "{:e}", f32::from(self))?;
        if self.grad().is_some() {
            write!(f, ", ∇: {:e}", self.grad().unwrap())?;
        }
        write!(f,"]")
    }
}

// Constructors
impl GradVal {
    fn from_op(v: f32, op: GradValOp) -> Self {
        GradVal {
            _gv: Rc::new(RefCell::new(Gv::from_op(v, op))),
        }
    }
}
impl From<f32> for GradVal {
    fn from(value: f32) -> Self {
        GradVal {
            _gv: Rc::new(RefCell::new(value.into())),
        }
    }
}
impl From<&GradVal> for f32 {
    fn from(gv: &GradVal) -> Self {
        gv._gv.borrow()._val
    }
}

impl PartialEq for GradVal {
    fn eq(&self, other: &Self) -> bool {
        RefCell::borrow(&self._gv)._val == RefCell::borrow(&other._gv)._val
    }
}

impl PartialOrd for GradVal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        RefCell::borrow(&self._gv)
            ._val
            .partial_cmp(&RefCell::borrow(&other._gv)._val)
    }
}

impl Neg for &GradVal {
    type Output = GradVal;

    fn neg(self) -> Self::Output {
        GradVal::from_op(
            -RefCell::borrow(&self._gv)._val,
            GradValOp::Neg(self._gv.clone()),
        )
    }
}

impl Add for &GradVal {
    type Output = GradVal;

    fn add(self, other: Self) -> Self::Output {
        GradVal {
            _gv: Rc::new(RefCell::new(Gv::from_op(
                RefCell::borrow(&self._gv)._val + RefCell::borrow(&other._gv)._val,
                GradValOp::Add(self._gv.clone(), other._gv.clone()),
            ))),
        }
    }
}

impl Sub for &GradVal {
    type Output = GradVal;

    fn sub(self, other: Self) -> Self::Output {
        GradVal {
            _gv: Rc::new(RefCell::new(Gv::from_op(
                RefCell::borrow(&self._gv)._val - RefCell::borrow(&other._gv)._val,
                GradValOp::Sub(self._gv.clone(), other._gv.clone()),
            ))),
        }
    }
}

impl Mul for &GradVal {
    type Output = GradVal;

    fn mul(self, other: Self) -> Self::Output {
        GradVal {
            _gv: Rc::new(RefCell::new(Gv::from_op(
                RefCell::borrow(&self._gv)._val * RefCell::borrow(&other._gv)._val,
                GradValOp::Mul(self._gv.clone(), other._gv.clone()),
            ))),
        }
    }
}

impl Div for &GradVal {
    type Output = GradVal;

    fn div(self, other: Self) -> Self::Output {
        let divider = RefCell::borrow(&other._gv)._val;
        if divider == 0. {
            panic!("Division by Zero :(");
        }
        GradVal {
            _gv: Rc::new(RefCell::new(Gv::from_op(
                RefCell::borrow(&self._gv)._val / divider,
                GradValOp::Div(self._gv.clone(), other._gv.clone()),
            ))),
        }
    }
}

// Additional operators
impl GradVal {
    pub fn exp(&self) -> Self {
        GradVal::from_op(f32::from(self).exp(), GradValOp::Exp(self._gv.clone()))
    }

    pub fn log(&self) -> Self {
        GradVal::from_op(f32::from(self).ln(), GradValOp::Log(self._gv.clone()))
    }

    pub fn pow(&self, other: &GradVal) -> Self {
        GradVal::from_op(
            f32::from(self).powf(other._gv.borrow()._val),
            GradValOp::Pow(self._gv.clone(), other._gv.clone()),
        )
    }

    pub fn powf(&self, other: f32) -> Self {
        let other = GradVal::from(other);
        return self.pow(&other);
    }
}

// Backward propagation
impl GradVal {
    pub fn backward(&mut self) {
        RefCell::borrow_mut(&self._gv).reset_grad_recursively();
        RefCell::borrow_mut(&self._gv).calc_grad_recursively(1.);
    }
}

// Access functions
impl GradVal {
    pub fn grad(&self) -> Option<f32> {
        self._gv.borrow()._grad
    }
}

#[cfg(test)]
mod tests;