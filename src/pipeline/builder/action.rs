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
        options =$options: ident,
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
                                trace!("Get");
                                $get(request, &setting, utility_thread)
                            },
                            Method::Head { .. } => {
                                trace!("Head");
                                $head(request, &setting, utility_thread)
                            },
                            Method::Post { .. } => {
                                trace!("Post");
                                $post(request, &setting, utility_thread)
                            },
                            Method::Put { .. } => {
                                trace!("Put");
                                $put(request, &setting, utility_thread)
                            },
                            Method::Delete { .. } => {
                                trace!("Delete");
                                $delete(request, &setting, utility_thread)
                            },
                            Method::Connect { .. } => {
                                trace!("Connect");
                                $connect(request, &setting, utility_thread)
                            },
                            Method::Options { .. } => {
                                trace!("Options");
                                $options(request, &setting, utility_thread)
                            },
                            Method::Trace { .. } => {
                                trace!("Trace");
                                $trace(request, &setting, utility_thread)
                            },
                            Method::Patch { .. } => {
                                trace!("Patch");
                                $patch(request, &setting, utility_thread)
                            },
                        }
                    },
                    Err(err_code) => {
                        trace!("Error");
                        return Ok(
                            $err(err_code, setting, utility_thread)
                        )
                    },
                }
            }
        }
    }
}
