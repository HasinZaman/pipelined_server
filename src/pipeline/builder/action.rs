#[macro_export]
macro_rules! ActionBuilder {
    (
        name = $name: ident,

        utility = $utility: ty,

        get = $get: ident,
        head = $head: ident,
        post = $post: ident,
        put = $put: ident,
        delete = $delete: ident,
        connect = $connect: ident,
        options = $options: ident,
        trace = $trace: ident,
        patch = $patch: ident,

        error = $err: ident
     ) => {
        use paste::paste;

        paste!{
            pub fn [<$name>] (request: &Result<Request, ResponseStatusCode>, setting: &ServerSetting, utility_thread: &mut $utility) -> Result<Response, ResponseStatusCode> {
                match request{
                    Ok(request) => {
                        match &request.0 {
                            Method::Get { .. } => {
                                trace!("Get:{request:#?}");
                                $get(request, &setting, utility_thread)
                            },
                            Method::Head { .. } => {
                                trace!("Head:{request:#?}");
                                $head(request, &setting, utility_thread)
                            },
                            Method::Post { .. } => {
                                trace!("Post:{request:#?}");
                                $post(request, &setting, utility_thread)
                            },
                            Method::Put { .. } => {
                                trace!("Put:{request:#?}");
                                $put(request, &setting, utility_thread)
                            },
                            Method::Delete { .. } => {
                                trace!("Delete:{request:#?}");
                                $delete(request, &setting, utility_thread)
                            },
                            Method::Connect { .. } => {
                                trace!("Connect:{request:#?}");
                                $connect(request, &setting, utility_thread)
                            },
                            Method::Options { .. } => {
                                trace!("Options:{request:#?}");
                                $options(request, &setting, utility_thread)
                            },
                            Method::Trace { .. } => {
                                trace!("Trace:{request:#?}");
                                $trace(request, &setting, utility_thread)
                            },
                            Method::Patch { .. } => {
                                trace!("Patch:{request:#?}");
                                $patch(request, &setting, utility_thread)
                            },
                        }
                    },
                    Err(err_code) => {
                        trace!("Error:{err_code:?}");
                        return Ok(
                            $err(err_code, setting, utility_thread)
                        )
                    },
                }
            }
        }
    }
}
