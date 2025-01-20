macro_rules! try_from_state_event_fn {
    ($fn:ident, $ty:ty) => {
        impl<E> TryFromState<Arc<ScreenReaderState>, E> for $ty
        where
            E: EventProperties,
        {
          type Error = OdiliaError;
          type Future = impl Future<Output = Result<Self, Self::Error>>;
          fn try_from_state(state: Arc<ScreenReaderState>, event: E) -> Self::Future {
              $fn(state, event)
          }
        }
    };
    ($fn:ident, $ty:ty, $($trait:path),+ $(,)?) => {
        impl<E> TryFromState<Arc<ScreenReaderState>, E> for $ty
        where
            E: EventProperties + $($trait +)+
        {
          type Error = OdiliaError;
          type Future = impl Future<Output = Result<Self, Self::Error>>;
          fn try_from_state(state: Arc<ScreenReaderState>, event: E) -> Self::Future {
              $fn(state, event)
          }
        }
    }
}
