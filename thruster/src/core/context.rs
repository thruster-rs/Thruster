use bytes::Bytes;

/// A `Context` is what will be passed between functions in the middleware for
/// the defined routes of Thruster. Since a new context is made for each
/// incomming request, it's important to keep this struct lean and quick, as
/// well as the `context_generator` associated with it.
pub trait Context {
    type Response: Send;

    /// get_response returns a fully created response object based on the contents
    /// of the Context. This means setting the body according to whatever has been
    /// stored via set_body and/or set_body_bytes, as well as adding the proper
    /// headers that have been added via the set method.
    fn get_response(self) -> Self::Response;

    /// set_body is used to set the body using a vec of bytes on the context. The
    /// contents will be used later for generating the correct response.
    fn set_body(&mut self, body: Vec<u8>);

    /// set_body_byte is used to set the body using a Bytes object on the context.
    /// The contents will be used later for generating the correct response.
    fn set_body_bytes(&mut self, bytes: Bytes);

    /// route is used to return the route from the incoming request as a string.
    fn route(&self) -> &str;

    /// set is used to set a header on the outgoing response.
    fn set(&mut self, key: &str, value: &str);

    /// remove is used to remove a header on the outgoing response.
    fn remove(&mut self, key: &str);
}
