use bitcoin::types::errors::Errors;
use num_traits::identities::Zero;
use num_bigint::BigInt;
use core::ops::Add;

#[allow(dead_code)]
#[derive(Clone, Debug)]
enum Point<const A: i64, const B: i64> {
    Point(BigInt, BigInt),
    Infinity
}

#[allow(dead_code)]
impl<const A: i64, const B:i64> Point<A, B> {
    fn new_point(x: BigInt, y: BigInt) -> Result<Self, Errors> {
        // Checks if point is included in the curve y2 = x3 + ax + b
        if y.pow(2) != x.pow(3) + A * &x + B {
            return Err(Errors::InvalidPoint);
        }

        Ok(Point::<A, B>::Point(x, y))
        }

    fn new_infinity() -> Self {
        Point::<A,B>::Infinity
    }
}

#[allow(dead_code)]
impl<const A: i64, const B: i64> PartialEq for Point<A, B> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Point::Point(x1, y1), Point::Point(x2, y2)) => x1 == x2 && y1 == y2,
            (Point::Infinity, Point::Infinity) => true,
            _ => false,
        }
    }
}

impl<const A: i64, const B: i64> Add<Point<A, B>> for Point<A, B> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self, other) {
            // If either operand is the identity (point at infinity), return the other.
            (Point::Infinity, p) => p,
            (p, Point::Infinity) => p,

            // Both are actual points on the curve.
            (Point::Point(x1, y1), Point::Point(x2, y2)) => {
                if x1 == x2 {
                    if y1 == y2 {
                        // ---- Doubling case (P1 == P2) ----
                        if y1.is_zero() {
                            // Tangent is vertical if y1 = 0, so result is infinity.
                            Point::new_infinity()
                        } else {
                            // slope = (3*x1^2 + A) / (2*y1)
                            let numerator  = BigInt::from(3) * x1.pow(2_u32) + BigInt::from(A);
                            let denominator = BigInt::from(2) * &y1;
                            let slope = numerator / denominator;

                            let x3 = slope.pow(2_u32) - (BigInt::from(2) * &x1);
                            let y3 = &slope * (&x1 - &x3) - &y1;
                            Point::new_point(x3, y3).unwrap()
                        }
                    } else {
                        // ---- P1 = -P2 => vertical line => infinity. ----
                        Point::new_infinity()
                    }
                } else {
                    // ---- Addition case (x1 != x2) ----
                    let slope = (&y2 - &y1) / (&x2 - &x1);
                    let x3 = slope.pow(2_u32) - &x1 - &x2;
                    let y3 = &slope * (&x1 - &x3) - &y1;
                    Point::new_point(x3, y3).unwrap()
                }
            }
        }
    }
}

#[cfg(test)]
mod point_tests {
    use num_bigint::ToBigInt;

    use super::*;

    #[test]
    fn test_create_valid_point() {
        assert!(Point::<5, 7>::new_point(-1.to_bigint().unwrap(),-1.to_bigint().unwrap()).is_ok());
    }

    #[test]
    fn test_create_valid_point_and_check_result() {
        let result = Point::<5, 7>::new_point(-1.to_bigint().unwrap(), -1.to_bigint().unwrap());
        assert!(result.is_ok());
    
        let point = result.unwrap();
        assert_eq!(point, Point::<5, 7>::Point(-1.to_bigint().unwrap(), -1.to_bigint().unwrap()));
    }

    #[test]
    fn test_create_valid_point_at_infinity() {
        assert_eq!(Point::<5,7>::new_infinity(), Point::<5,7>::Infinity);
    }

    #[test]
    fn test_eq() {
        assert!(Point::<5,7>::new_infinity() == Point::<5,7>::Infinity);
        assert!(Point::<5, 7>::Point(-1.to_bigint().unwrap(), -1.to_bigint().unwrap()) == Point::<5, 7>::Point(-1.to_bigint().unwrap(), -1.to_bigint().unwrap()));
        assert!(Point::<5, 7>::Point(-1.to_bigint().unwrap(), -1.to_bigint().unwrap()) != Point::<5, 7>::Point(-1.to_bigint().unwrap(), 1.to_bigint().unwrap())); 
        assert!(Point::<5, 7>::Infinity != Point::<5, 7>::Point(-1.to_bigint().unwrap(), 1.to_bigint().unwrap()));  
    } 

    #[test]
    fn test_add_infinity_to_point() {
        let infinity = Point::<5,7>::new_infinity();
        let point = Point::<5,7>::new_point(-1.to_bigint().unwrap(),-1.to_bigint().unwrap()).unwrap();

        assert_eq!(infinity + point, Point::<5,7>::new_point(-1.to_bigint().unwrap(),-1.to_bigint().unwrap()).unwrap());
    }

    #[test]
    fn test_add_infinity_to_point_reverse() {
        let infinity = Point::<5,7>::new_infinity();
        let point = Point::<5,7>::new_point(-1.to_bigint().unwrap(),-1.to_bigint().unwrap()).unwrap();

        assert_eq!(point + infinity, Point::<5,7>::new_point(-1.to_bigint().unwrap(),-1.to_bigint().unwrap()).unwrap());
    }

    #[test]
    fn test_add_vertical_line() {
        // This happen when points have same x and different y coordinates
        let point1 = Point::<5,7>::new_point(-1.to_bigint().unwrap(),1.to_bigint().unwrap()).unwrap(); 
        let point2 = Point::<5,7>::new_point(-1.to_bigint().unwrap(),-1.to_bigint().unwrap()).unwrap();

        assert_eq!(point1 + point2, Point::<5,7>::new_infinity());
    }

    #[test]
    fn test_add_same_point_with_vertical_slope() {
        // This happen when points are the same and have y == 0
        let point1 = Point::<0,0>::new_point(0.to_bigint().unwrap(),0.to_bigint().unwrap()).unwrap(); 
        let point2 = Point::<0,0>::new_point(0.to_bigint().unwrap(),0.to_bigint().unwrap()).unwrap();

        assert!(point1 == point2);
        assert_eq!(point1 + point2, Point::<0,0>::new_infinity());
    }

    #[test]
    fn test_add_same_point() {
        // p(-1,-1) + p(-1,-1) = p(18,77)
        let point1 = Point::<5,7>::new_point(-1.to_bigint().unwrap(),-1.to_bigint().unwrap()).unwrap(); 
        let point2 = Point::<5,7>::new_point(-1.to_bigint().unwrap(),-1.to_bigint().unwrap()).unwrap();

        assert!(point1 == point2);
        assert_eq!(point1 + point2, Point::<5,7>::new_point(18.to_bigint().unwrap(), 77.to_bigint().unwrap()).unwrap());
    }

    #[test]
    fn test_add_points_with_different_x() {
        // p(2,5) + p(-1,-1) = p(3,-7)
        let point1 = Point::<5,7>::new_point(2.to_bigint().unwrap(),5.to_bigint().unwrap()).unwrap(); 
        let point2 = Point::<5,7>::new_point(-1.to_bigint().unwrap(),-1.to_bigint().unwrap()).unwrap();

        assert!(point1 != point2);
        assert_eq!(point1 + point2, Point::<5,7>::new_point(3.to_bigint().unwrap(), -7.to_bigint().unwrap()).unwrap());
    }
}