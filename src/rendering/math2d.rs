use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(Default, Debug, Clone, Copy)]
pub struct Rect<X = f32, Y = f32> {
    pub left:   X,
    pub top:    Y,
    pub right:  X,
    pub bottom: Y,
}
impl<X, Y> Rect<X, Y> {
    pub fn from_ranges(x: std::ops::Range<X>, y: std::ops::Range<Y>) -> Self {
        Rect {
            left:   x.start,
            right:  x.end,
            top:    y.start,
            bottom: y.end,
        }
    }
}
impl<X: Copy + Sub, Y: Copy + Sub> Rect<X, Y> {
    pub fn width(&self) -> X::Output { self.right - self.left }
    pub fn height(&self) -> Y::Output { self.bottom - self.top }
    pub fn size(&self) -> Vec2<X::Output, Y::Output> {
        Vec2 {
            x: self.width(),
            y: self.height(),
        }
    }
}
impl<X: Copy, Y: Copy> Rect<X, Y> {
    pub fn x_range(&self) -> std::ops::RangeInclusive<X> { self.left..=self.right }
    pub fn y_range(&self) -> std::ops::RangeInclusive<Y> { self.top..=self.bottom }
    pub fn min(&self) -> Vec2<X, Y> { self.tl() }
    pub fn max(&self) -> Vec2<X, Y> { self.br() }
    pub fn tl(&self) -> Vec2<X, Y> {
        Vec2 {
            x: self.left,
            y: self.top,
        }
    }
    pub fn tr(&self) -> Vec2<X, Y> {
        Vec2 {
            x: self.right,
            y: self.top,
        }
    }
    pub fn bl(&self) -> Vec2<X, Y> {
        Vec2 {
            x: self.left,
            y: self.bottom,
        }
    }
    pub fn br(&self) -> Vec2<X, Y> {
        Vec2 {
            x: self.right,
            y: self.bottom,
        }
    }
}

impl<X: Lerp, Y: Lerp> Rect<X, Y> {
    pub fn remap<X2: Lerp, Y2: Lerp>(self, from: Rect<X, Y>, to: Rect<X2, Y2>) -> Rect<X2, Y2> {
        Rect {
            left:   self.left.remap(from.x_range(), to.x_range()),
            right:  self.right.remap(from.x_range(), to.x_range()),
            top:    self.top.remap(from.y_range(), to.y_range()),
            bottom: self.bottom.remap(from.y_range(), to.y_range()),
        }
    }
    pub fn zoom(self, factor: Vec2<f32, f32>, pivot: Vec2<X, Y>) -> Self {
        Rect {
            left:   X::lerp(pivot.x..=self.left, factor.x),
            right:  X::lerp(pivot.x..=self.right, factor.x),
            top:    Y::lerp(pivot.y..=self.top, factor.y),
            bottom: Y::lerp(pivot.y..=self.bottom, factor.y),
        }
    }
    pub fn zoom_by_clicks(self, clicks: Vec2<f32, f32>, pivot: Vec2<X, Y>) -> Self {
        self.zoom(
            Vec2 {
                x: 2f32.powf(clicks.x),
                y: 2f32.powf(clicks.y),
            },
            pivot,
        )
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Vec2<X = f32, Y = f32> {
    pub x: X,
    pub y: Y,
}
impl<X: Lerp, Y: Lerp> Vec2<X, Y> {
    pub fn remap<X2: Lerp, Y2: Lerp>(self, from: Rect<X, Y>, to: Rect<X2, Y2>) -> Vec2<X2, Y2> {
        Vec2 {
            x: self.x.remap(from.x_range(), to.x_range()),
            y: self.y.remap(from.y_range(), to.y_range()),
        }
    }
}
pub fn vec2<X, Y>(x: X, y: Y) -> Vec2<X, Y> { Vec2 { x, y } }

pub trait Lerp: Copy {
    fn lerp(range: std::ops::RangeInclusive<Self>, t: f32) -> Self;
    fn inverse_lerp(self, range: std::ops::RangeInclusive<Self>) -> f32;
    fn remap<T: Lerp>(
        self,
        from: std::ops::RangeInclusive<Self>,
        to: std::ops::RangeInclusive<T>,
    ) -> T {
        T::lerp(to, self.inverse_lerp(from))
    }
}
impl Lerp for f32 {
    fn lerp(range: std::ops::RangeInclusive<Self>, t: f32) -> Self {
        range.start() + t * (range.end() - range.start())
    }
    fn inverse_lerp(self, range: std::ops::RangeInclusive<Self>) -> f32 {
        (self - range.start()) / (range.end() - range.start())
    }
}

macro_rules! impl_binop {
    ($Struct:ident { $($field:ident),+ }, $Op:ident, $op:ident, $OpAssign:ident, $op_assign:ident) => {
        impl<X: $Op<X2>, Y: $Op<Y2>, X2, Y2> $Op<$Struct<X2, Y2>> for $Struct<X, Y> {
            type Output = $Struct<X::Output, Y::Output>;
            fn $op(self, rhs: $Struct<X2, Y2>) -> Self::Output {
                $Struct { $($field: self.$field.$op(rhs.$field)),+ }
            }
        }
        impl<X: $OpAssign<X2>, Y: $OpAssign<Y2>, X2, Y2> $OpAssign<$Struct<X2, Y2>> for $Struct<X, Y> {
            fn $op_assign(&mut self, rhs: $Struct<X2, Y2>) {
                $(self.$field.$op_assign(rhs.$field);)+
            }
        }
    };
}

macro_rules! impl_unop {
    ($Struct:ident { $($field:ident),+ }, $Op:ident, $op:ident) => {
        impl<X: $Op, Y: $Op> $Op for $Struct<X, Y> {
            type Output = $Struct<X::Output, Y::Output>;
            fn $op(self) -> Self::Output {
                $Struct { $($field: self.$field.$op()),+ }
            }
        }
    };
}

macro_rules! impl_math_ops {
    ($Struct:ident { $($field:ident),+ }) => {
        impl_binop!($Struct { $($field),+ }, Add, add, AddAssign, add_assign);
        impl_binop!($Struct { $($field),+ }, Sub, sub, SubAssign, sub_assign);
        impl_binop!($Struct { $($field),+ }, Mul, mul, MulAssign, mul_assign);
        impl_binop!($Struct { $($field),+ }, Div, div, DivAssign, div_assign);
        impl_unop!($Struct { $($field),+ }, Neg, neg);
    };
}

impl_math_ops!(Vec2 { x, y });
impl_math_ops!(Rect {
    left,
    top,
    right,
    bottom
});

macro_rules! impl_rect_vec2_binop {
    ($Op:ident, $op:ident, $OpAssign:ident, $op_assign:ident) => {
        impl<X: $Op<X2>, Y: $Op<Y2>, X2: Copy, Y2: Copy> $Op<Vec2<X2, Y2>> for Rect<X, Y> {
            type Output = Rect<X::Output, Y::Output>;
            fn $op(self, rhs: Vec2<X2, Y2>) -> Self::Output {
                Rect {
                    left:   self.left.$op(rhs.x),
                    right:  self.right.$op(rhs.x),
                    top:    self.top.$op(rhs.y),
                    bottom: self.bottom.$op(rhs.y),
                }
            }
        }
        impl<X: $OpAssign<X2>, Y: $OpAssign<Y2>, X2: Copy, Y2: Copy> $OpAssign<Vec2<X2, Y2>>
            for Rect<X, Y>
        {
            fn $op_assign(&mut self, rhs: Vec2<X2, Y2>) {
                self.left.$op_assign(rhs.x);
                self.right.$op_assign(rhs.x);
                self.top.$op_assign(rhs.y);
                self.bottom.$op_assign(rhs.y);
            }
        }
    };
}
impl_rect_vec2_binop!(Add, add, AddAssign, add_assign);
impl_rect_vec2_binop!(Sub, sub, SubAssign, sub_assign);
impl_rect_vec2_binop!(Mul, mul, MulAssign, mul_assign);
impl_rect_vec2_binop!(Div, div, DivAssign, div_assign);
