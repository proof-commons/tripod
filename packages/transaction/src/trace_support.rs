pub trait TraceRefusal {
    fn tracing_kind(&self) -> &'static str;
}

impl<T: TraceRefusal + ?Sized> TraceRefusal for Box<T> {
    fn tracing_kind(&self) -> &'static str {
        (**self).tracing_kind()
    }
}
