mod http_body_enum_test {

    mod enum_to_string_test {
        use crate::http::body::{Application, Audio, ContentType, Image, Multipart, Text, Video};

        #[test]
        fn application_test() {
            {
                let actual: ContentType = ContentType::Application(Application::EDI_X12);
                assert_eq!(actual.to_string(), "application/EDI-X12");
            }

            {
                let actual: ContentType = ContentType::Application(Application::EDIFACT);
                assert_eq!(actual.to_string(), "application/EDIFACT");
            }

            {
                let actual: ContentType = ContentType::Application(Application::javascript);
                assert_eq!(actual.to_string(), "application/javascript");
            }

            {
                let actual: ContentType = ContentType::Application(Application::octet_stream);
                assert_eq!(actual.to_string(), "application/octet-stream");
            }

            {
                let actual: ContentType = ContentType::Application(Application::ogg);
                assert_eq!(actual.to_string(), "application/ogg");
            }

            {
                let actual: ContentType = ContentType::Application(Application::pdf);
                assert_eq!(actual.to_string(), "application/pdf");
            }

            {
                let actual: ContentType = ContentType::Application(Application::xhtml_xml);
                assert_eq!(actual.to_string(), "application/xhtml+xml");
            }

            {
                let actual: ContentType = ContentType::Application(Application::x_shockwave_flash);
                assert_eq!(actual.to_string(), "application/x-shockwave-flash");
            }

            {
                let actual: ContentType = ContentType::Application(Application::json);
                assert_eq!(actual.to_string(), "application/json");
            }

            {
                let actual: ContentType = ContentType::Application(Application::ld_json);
                assert_eq!(actual.to_string(), "application/ld+json");
            }

            {
                let actual: ContentType = ContentType::Application(Application::xml);
                assert_eq!(actual.to_string(), "application/xml");
            }

            {
                let actual: ContentType = ContentType::Application(Application::zip);
                assert_eq!(actual.to_string(), "application/zip");
            }

            {
                let actual: ContentType =
                    ContentType::Application(Application::x_www_form_urlencoded);
                assert_eq!(actual.to_string(), "application/x-www-form-urlencoded");
            }
        }

        #[test]
        fn audio_test() {
            {
                let actual: ContentType = ContentType::Audio(Audio::mpeg);
                assert_eq!(actual.to_string(), "audio/mpeg");
            }

            {
                let actual: ContentType = ContentType::Audio(Audio::x_ms_wma);
                assert_eq!(actual.to_string(), "audio/x-ms-wma");
            }

            {
                let actual: ContentType = ContentType::Audio(Audio::vnd_rn_realaudio);
                assert_eq!(actual.to_string(), "audio/vnd.rn-realaudio");
            }

            {
                let actual: ContentType = ContentType::Audio(Audio::x_wav);
                assert_eq!(actual.to_string(), "audio/x-wav");
            }
        }

        #[test]
        fn image_test() {
            {
                let actual: ContentType = ContentType::Image(Image::gif);
                assert_eq!(actual.to_string(), "image/gif");
            }

            {
                let actual: ContentType = ContentType::Image(Image::jpeg);
                assert_eq!(actual.to_string(), "image/jpeg");
            }

            {
                let actual: ContentType = ContentType::Image(Image::png);
                assert_eq!(actual.to_string(), "image/png");
            }

            {
                let actual: ContentType = ContentType::Image(Image::tiff);
                assert_eq!(actual.to_string(), "image/tiff");
            }

            {
                let actual: ContentType = ContentType::Image(Image::vnd_microsoft_icon);
                assert_eq!(actual.to_string(), "image/vnd.microsoft.icon");
            }

            {
                let actual: ContentType = ContentType::Image(Image::x_icon);
                assert_eq!(actual.to_string(), "image/x-icon");
            }

            {
                let actual: ContentType = ContentType::Image(Image::vnd_djvu);
                assert_eq!(actual.to_string(), "image/vnd.djvu");
            }

            {
                let actual: ContentType = ContentType::Image(Image::svg_xml);
                assert_eq!(actual.to_string(), "image/svg+xml");
            }
        }

        #[test]
        fn multipart_test() {
            {
                let actual: ContentType = ContentType::Multipart(Multipart::mixed);
                assert_eq!(actual.to_string(), "multipart/mixed");
            }

            {
                let actual: ContentType = ContentType::Multipart(Multipart::alternative);
                assert_eq!(actual.to_string(), "multipart/alternative");
            }

            {
                let actual: ContentType = ContentType::Multipart(Multipart::related);
                assert_eq!(actual.to_string(), "multipart/related");
            }

            // {
            //     let actual: ContentType = ContentType::Multipart(Multipart::form_data {
            //         boundary: String::from(""),
            //     });
            //     assert_eq!(actual.to_string(), "multipart/form-data");
            // }
        }

        #[test]
        fn text_test() {
            {
                let actual: ContentType = ContentType::Text(Text::css);
                assert_eq!(actual.to_string(), "text/css");
            }

            {
                let actual: ContentType = ContentType::Text(Text::csv);
                assert_eq!(actual.to_string(), "text/csv");
            }

            {
                let actual: ContentType = ContentType::Text(Text::html);
                assert_eq!(actual.to_string(), "text/html");
            }

            {
                let actual: ContentType = ContentType::Text(Text::javascript);
                assert_eq!(actual.to_string(), "text/javascript");
            }

            {
                let actual: ContentType = ContentType::Text(Text::plain);
                assert_eq!(actual.to_string(), "text/plain");
            }

            {
                let actual: ContentType = ContentType::Text(Text::xml);
                assert_eq!(actual.to_string(), "text/xml");
            }
        }

        #[test]
        fn video_test() {
            {
                let actual: ContentType = ContentType::Video(Video::mpeg);
                assert_eq!(actual.to_string(), "video/mpeg");
            }

            {
                let actual: ContentType = ContentType::Video(Video::mp4);
                assert_eq!(actual.to_string(), "video/mp4");
            }

            {
                let actual: ContentType = ContentType::Video(Video::quicktime);
                assert_eq!(actual.to_string(), "video/quicktime");
            }

            {
                let actual: ContentType = ContentType::Video(Video::x_ms_wmv);
                assert_eq!(actual.to_string(), "video/x-ms-wmv");
            }

            {
                let actual: ContentType = ContentType::Video(Video::x_msvideo);
                assert_eq!(actual.to_string(), "video/x-msvideo");
            }

            {
                let actual: ContentType = ContentType::Video(Video::x_flv);
                assert_eq!(actual.to_string(), "video/x-flv");
            }

            {
                let actual: ContentType = ContentType::Video(Video::webm);
                assert_eq!(actual.to_string(), "video/webm");
            }
        }
    }

    mod string_to_enum_test {
        use crate::http::body::{Application, Audio, ContentType, Image, Multipart, Text, Video};

        fn string_to_enum_test(input_str: &str, expected: ContentType) {
            let actual = ContentType::new(input_str);

            match actual {
                Ok(actual) => {
                    assert_eq!(actual.to_string(), expected.to_string());
                }
                _ => panic!("Request::from_str Error"),
            }
        }

        #[test]
        fn application_edi_x12_test() {
            string_to_enum_test(
                "application/EDI-X12",
                ContentType::Application(Application::EDI_X12),
            );
        }

        #[test]
        fn application_edifact_test() {
            string_to_enum_test(
                "application/EDIFACT",
                ContentType::Application(Application::EDIFACT),
            );
        }

        #[test]
        fn application_javascript_test() {
            string_to_enum_test(
                "application/javascript",
                ContentType::Application(Application::javascript),
            );
        }

        #[test]
        fn application_octet_stream_test() {
            string_to_enum_test(
                "application/octet-stream",
                ContentType::Application(Application::octet_stream),
            );
        }

        #[test]
        fn application_ogg_test() {
            string_to_enum_test(
                "application/ogg",
                ContentType::Application(Application::ogg),
            );
        }

        #[test]
        fn application_pdf_test() {
            string_to_enum_test(
                "application/pdf",
                ContentType::Application(Application::pdf),
            );
        }

        #[test]
        fn application_xhtml_xml_test() {
            string_to_enum_test(
                "application/xhtml+xml",
                ContentType::Application(Application::xhtml_xml),
            );
        }

        #[test]
        fn application_x_shockwave_flash_test() {
            string_to_enum_test(
                "application/x-shockwave-flash",
                ContentType::Application(Application::x_shockwave_flash),
            );
        }

        #[test]
        fn application_json_test() {
            string_to_enum_test(
                "application/json",
                ContentType::Application(Application::json),
            );
        }

        #[test]
        fn application_ld_json_test() {
            string_to_enum_test(
                "application/ld+json",
                ContentType::Application(Application::ld_json),
            );
        }

        #[test]
        fn application_zip_test() {
            string_to_enum_test(
                "application/zip",
                ContentType::Application(Application::zip),
            );
        }

        #[test]
        fn application_x_www_form_urlencoded_test() {
            string_to_enum_test(
                "application/x-www-form-urlencoded",
                ContentType::Application(Application::x_www_form_urlencoded),
            );
        }

        #[test]
        fn audio_mpeg_test() {
            string_to_enum_test("audio/mpeg", ContentType::Audio(Audio::mpeg));
        }

        #[test]
        fn audio_vnd_rn_realaudio_test() {
            string_to_enum_test(
                "audio/vnd.rn-realaudio",
                ContentType::Audio(Audio::vnd_rn_realaudio),
            );
        }

        #[test]
        fn audio_x_wav_test() {
            string_to_enum_test("audio/x-wav", ContentType::Audio(Audio::x_wav));
        }

        #[test]
        fn image_gif_test() {
            string_to_enum_test("image/gif", ContentType::Image(Image::gif));
        }

        #[test]
        fn image_jpeg_test() {
            string_to_enum_test("image/jpeg", ContentType::Image(Image::jpeg));
        }

        #[test]
        fn image_png_test() {
            string_to_enum_test("image/png", ContentType::Image(Image::png));
        }

        #[test]
        fn image_tiff_test() {
            string_to_enum_test("image/tiff", ContentType::Image(Image::tiff));
        }

        #[test]
        fn image_vnd_microsoft_icon_test() {
            string_to_enum_test(
                "image/vnd.microsoft.icon",
                ContentType::Image(Image::vnd_microsoft_icon),
            );
        }

        #[test]
        fn image_x_icon_test() {
            string_to_enum_test("image/x-icon", ContentType::Image(Image::x_icon));
        }

        #[test]
        fn image_vnd_djvu_test() {
            string_to_enum_test("image/vnd.djvu", ContentType::Image(Image::vnd_djvu));
        }

        #[test]
        fn image_svg_xml_test() {
            string_to_enum_test("image/svg+xml", ContentType::Image(Image::svg_xml));
        }

        #[test]
        fn multipart_mixed_test() {
            string_to_enum_test("multipart/mixed", ContentType::Multipart(Multipart::mixed));
        }

        #[test]
        fn multipart_alternative_test() {
            string_to_enum_test(
                "multipart/alternative",
                ContentType::Multipart(Multipart::alternative),
            );
        }

        #[test]
        fn multipart_related_test() {
            string_to_enum_test(
                "multipart/related",
                ContentType::Multipart(Multipart::related),
            );
        }

        // #[test]
        // fn multipart_form_data_test() {
        //     string_to_enum_test(
        //         "multipart/form-data",
        //         ContentType::Multipart(Multipart::form_data {
        //             boundary: String::from(""),
        //         }),
        //     );
        // }

        #[test]
        fn text_css_test() {
            string_to_enum_test("text/css", ContentType::Text(Text::css));
        }

        #[test]
        fn text_csv_test() {
            string_to_enum_test("text/csv", ContentType::Text(Text::csv));
        }

        #[test]
        fn text_html_test() {
            string_to_enum_test("text/html", ContentType::Text(Text::html));
        }

        #[test]
        fn text_javascript_test() {
            string_to_enum_test("text/javascript", ContentType::Text(Text::javascript));
        }

        #[test]
        fn text_plain_test() {
            string_to_enum_test("text/plain", ContentType::Text(Text::plain));
        }

        #[test]
        fn text_xml_test() {
            string_to_enum_test("text/xml", ContentType::Text(Text::xml));
        }

        #[test]
        fn video_mpeg_test() {
            string_to_enum_test("video/mpeg", ContentType::Video(Video::mpeg));
        }

        #[test]
        fn video_mp4_test() {
            string_to_enum_test("video/mp4", ContentType::Video(Video::mp4));
        }

        #[test]
        fn video_quicktime_test() {
            string_to_enum_test("video/quicktime", ContentType::Video(Video::quicktime));
        }

        #[test]
        fn video_x_ms_wmv_test() {
            string_to_enum_test("video/x-ms-wmv", ContentType::Video(Video::x_ms_wmv));
        }

        #[test]
        fn video_x_msvideo_test() {
            string_to_enum_test("video/x-msvideo", ContentType::Video(Video::x_msvideo));
        }

        #[test]
        fn video_x_flv_test() {
            string_to_enum_test("video/x-flv", ContentType::Video(Video::x_flv));
        }

        #[test]
        fn video_x_webm_test() {
            string_to_enum_test("video/webm", ContentType::Video(Video::webm));
        }
    }
}
mod make_http_request {
    use std::str::FromStr;

    use crate::http::{method::Method, request::Request};

    #[test]
    fn get_request() {
        //test modeled from syntax form https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/GET
        //GET /index.html

        let Request(method, _meta_data) = match Request::from_str("GET /index.html HTTP/1.1") {
            Ok(val) => val,
            Err(err) => panic!("{:?}", err),
        };

        match method {
            Method::Get { file } => {
                assert_eq!(file, String::from("/index.html"));
            }
            _ => {
                panic!("Incorrect variant. Got {} instead", method.to_string());
            }
        }
    }

    //head test
    #[test]
    fn head_request() {
        //test modeled from syntax form https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/HEAD
        //HEAD /index.html

        let Request(method, _meta_data) = match Request::from_str("HEAD /index.html HTTP/1.1") {
            Ok(val) => val,
            Err(err) => panic!("{:?}", err),
        };

        match method {
            Method::Head { file } => {
                assert_eq!(file, String::from("/index.html"));
            }
            _ => {
                panic!("Incorrect variant. Got {} instead", method.to_string());
            }
        }
    }

    //post test
    #[test]
    fn post_request() {
        //test modeled from syntax form https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/POST

        let Request(method, _meta_data) = match Request::from_str("POST /test HTTP/1.1\nHost: foo.example\nContent-Type: application/x-www-form-urlencoded\nContent-Length: 27\n\nfield1=value1&field2=value2") {
            Ok(val) => val,
            Err(err) => panic!("{:?}", err),
        };

        match method {
            Method::Post { file, body } => {
                assert_eq!(file, String::from("/test"));

                assert_eq!(
                    body.content_type.to_string(),
                    "application/x-www-form-urlencoded"
                );

                assert_eq!(body.content, "field1=value1&field2=value2".as_bytes());
            }
            _ => {
                panic!("Incorrect variant. Got {} instead", method.to_string());
            }
        }
    }

    //put test
    #[test]
    fn put_request() {
        //test modeled from syntax form https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/PUT

        let Request(method, _meta_data) = match Request::from_str("PUT /new.html HTTP/1.1\nHost: example.com\nContent-type: text/html\nContent-length: 16\n\n<p>New File</p>") {
            Ok(val) => val,
            Err(err) => panic!("{:?}", err),
        };

        match method {
            Method::Put { file, body } => {
                assert_eq!(file, String::from("/new.html"));

                assert_eq!(body.content_type.to_string(), "text/html");

                assert_eq!(body.content, "<p>New File</p>".as_bytes());
            }
            _ => {
                panic!("Incorrect variant. Got {} instead", method.to_string());
            }
        }
    }

    //delete test
    #[test]
    fn delete_request() {
        //test modeled from syntax form https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/DELETE

        let Request(method, _meta_data) = match Request::from_str("DELETE /file.html HTTP/1.1\nHost: example.com\nContent-type: text/html\nContent-length: 16\n\n<p>New File</p>") {
            Ok(val) => val,
            Err(err) => panic!("{:?}", err),
        };

        match method {
            Method::Delete { file, body } => {
                assert_eq!(file, String::from("/file.html"));

                match body {
                    Some(body) => {
                        assert_eq!(body.content_type.to_string(), "text/html");

                        assert_eq!(body.content, "<p>New File</p>".as_bytes());
                    }
                    None => {
                        panic!("Missing body");
                    }
                }
            }
            _ => {
                panic!("Incorrect variant. Got {} instead", method.to_string());
            }
        }
    }

    #[test]
    fn delete_no_body_request() {
        //test modeled from syntax form https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/DELETE

        let Request(method, _meta_data) =
            match Request::from_str("DELETE /file.html HTTP/1.1\nHost: example.com") {
                Ok(val) => val,
                Err(err) => panic!("{:?}", err),
            };

        match method {
            Method::Delete { file, body } => {
                assert_eq!(file, String::from("/file.html"));

                assert!(body.is_none());
            }
            _ => {
                panic!("Incorrect variant. Got {} instead", method.to_string());
            }
        }
    }

    //connect
    #[test]
    fn connect_request() {
        //test modeled from syntax form https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/CONNECT

        let Request(method, _meta_data) =
            match Request::from_str("CONNECT www.example.com:443 HTTP/1.1") {
                Ok(val) => val,
                Err(err) => panic!("{:?}", err),
            };

        match method {
            Method::Connect { url } => {
                assert_eq!(url, String::from("www.example.com:443"));
            }
            _ => {
                panic!("Incorrect variant. Got {} instead", method.to_string());
            }
        }
    }

    //options test
    #[test]
    fn options_request() {
        //test modeled from syntax form https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/OPTIONS

        let Request(method, _meta_data) = match Request::from_str("OPTIONS https://example.org -i")
        {
            Ok(val) => val,
            Err(err) => panic!("{:?}", err),
        };

        match method {
            Method::Options { url } => {
                assert_eq!(url, String::from("https://example.org"));
            }
            _ => {
                panic!("Incorrect variant. Got {} instead", method.to_string());
            }
        }
    }

    //trace test
    #[test]
    fn trace_request() {
        //test modeled from syntax form https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/OPTIONS

        let Request(method, _meta_data) = match Request::from_str("TRACE /index.html HTTP/1.1") {
            Ok(val) => val,
            Err(err) => panic!("{:?}", err),
        };

        match method {
            Method::Trace { file } => {
                assert_eq!(file, String::from("/index.html"));
            }
            _ => {
                panic!("Incorrect variant. Got {} instead", method.to_string());
            }
        }
    }

    //patch test
    #[test]
    fn patch_request() {
        //test modeled from syntax form https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/PUT

        let Request(method, _meta_data) = match Request::from_str("PATCH /file.txt HTTP/1.1\nHost: www.example.com\nContent-Type: application/pdf\nIf-Match: 'e0023aa4e'\nContent-Length: 100\n\n[description of changes]") {
            Ok(val) => val,
            Err(err) => panic!("{:?}", err),
        };

        match method {
            Method::Patch { file, body } => {
                assert_eq!(file, String::from("/file.txt"));

                assert_eq!(body.content_type.to_string(), "application/pdf");

                assert_eq!(body.content, "[description of changes]".as_bytes());
            }
            _ => {
                panic!("Incorrect variant. Got {} instead", method.to_string());
            }
        }
    }
}
