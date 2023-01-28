use either::Either;

pub fn either_of<L, R>(left: Option<L>, right: Option<R>) -> Option<Either<L, R>> {
    match (left, right) {
        (Some(l), _) => Some(Either::Left(l)),
        (_, Some(r)) => Some(Either::Right(r)),
        _ => None,
    }
}