use serde_json::Value;

macro_rules! json_safe {
    // Objeto com chaves como identificadores: { foo: 1, bar: 2 }
    ({ $($key:ident : $value:tt),* $(,)? }) => {{
        (|| -> ::serde_json::Result<::serde_json::Value> {
            let mut map = ::serde_json::Map::new();
            $(
                map.insert(
                    ::std::string::String::from(::std::stringify!($key)),
                    json_safe!($value)?,
                );
            )*
            ::std::result::Result::Ok(::serde_json::Value::Object(map))
        })()
    }};

    // Objeto com chaves literais: { "foo": 1, "bar": 2 }
    ({ $($key:literal : $value:tt),* $(,)? }) => {{
        (|| -> ::serde_json::Result<::serde_json::Value> {
            let mut map = ::serde_json::Map::new();
            $(
                map.insert(
                    ::std::string::String::from($key),
                    json_safe!($value)?,
                );
            )*
            ::std::result::Result::Ok(::serde_json::Value::Object(map))
        })()
    }};

    // Array: [ a, b, c ]
    ([ $($elem:tt),* $(,)? ]) => {{
        (|| -> ::serde_json::Result<::serde_json::Value> {
            let mut vec = ::std::vec::Vec::new();
            $(
                vec.push(json_safe!($elem)?);
            )*
            ::std::result::Result::Ok(::serde_json::Value::Array(vec))
        })()
    }};

    // null
    (null) => {{
        (|| -> ::serde_json::Result<::serde_json::Value> {
            ::std::result::Result::Ok(::serde_json::Value::Null)
        })()
    }};

    // Qualquer outra expressão vira serde_json::Value via to_value
    ($other:expr) => {
        ::serde_json::to_value($other)
    };
}

fn main() {
    // =========
    // 1) Objeto com chaves como identificadores
    // =========
    let obj_ident = json_safe!({
        foo: 1,
        bar: "baz",
        nested: {
            answer: 42,
        },
    });

    match obj_ident {
        Ok(Value::Object(map)) => {
            assert_eq!(map.get("foo"), Some(&Value::from(1)));
            assert_eq!(map.get("bar"), Some(&Value::from("baz")));

            match map.get("nested") {
                Some(Value::Object(nested)) => {
                    assert_eq!(nested.get("answer"), Some(&Value::from(42)));
                }
                _ => panic!("campo 'nested' não é objeto ou não existe"),
            }
        }
        Ok(_) => panic!("obj_ident não é um objeto JSON"),
        Err(e) => panic!("json_safe! com chaves ident retornou erro: {e}"),
    }

    // =========
    // 2) Objeto com chaves literais (&str)
    // =========
    let obj_literal = json_safe!({
        "user": "alice",
        "ativo": true,
        "contador": 10u64,
    });

    match obj_literal {
        Ok(Value::Object(map)) => {
            assert_eq!(map.get("user"), Some(&Value::from("alice")));
            assert_eq!(map.get("ativo"), Some(&Value::from(true)));
            assert_eq!(map.get("contador"), Some(&Value::from(10u64)));
        }
        Ok(_) => panic!("obj_literal não é um objeto JSON"),
        Err(e) => panic!("json_safe! com chaves literais retornou erro: {e}"),
    }

    // =========
    // 3) Array
    // =========
    let arr = json_safe!([
        1,
        "dois",
        false,
        { inner: "ok" },
        null,
    ]);

    match arr {
        Ok(Value::Array(vec)) => {
            assert_eq!(vec.len(), 5);
            assert_eq!(vec[0], Value::from(1));
            assert_eq!(vec[1], Value::from("dois"));
            assert_eq!(vec[2], Value::from(false));

            match &vec[3] {
                Value::Object(inner) => {
                    assert_eq!(inner.get("inner"), Some(&Value::from("ok")));
                }
                _ => panic!("elemento 3 do array não é objeto"),
            }

            assert!(matches!(vec[4], Value::Null));
        }
        Ok(_) => panic!("arr não é um array JSON"),
        Err(e) => panic!("json_safe! com array retornou erro: {e}"),
    }

    // =========
    // 4) null literal da macro
    // =========
    let n = json_safe!(null);

    match n {
        Ok(Value::Null) => {}
        Ok(_) => panic!("null da macro não resultou em Value::Null"),
        Err(e) => panic!("json_safe!(null) retornou erro: {e}"),
    }

    // =========
    // 5) Expressão genérica -> to_value (branch $other:expr)
    // =========
    #[derive(serde::Serialize)]
    struct MyStruct {
        id: u32,
        name: String,
    }

    let my_struct = MyStruct {
        id: 7,
        name: "bob".to_string(),
    };

    let expr_res = json_safe!(my_struct);

    match expr_res {
        Ok(Value::Object(map)) => {
            assert_eq!(map.get("id"), Some(&Value::from(7)));
            assert_eq!(map.get("name"), Some(&Value::from("bob")));
        }
        Ok(_) => panic!("expressão genérica não virou objeto como esperado"),
        Err(e) => panic!("json_safe!(my_struct) retornou erro: {e}"),
    }

    // =========
    // 6) Composição aninhada de todos os tipos
    // =========
    let complex = json_safe!({
        meta: {
            versao: 1,
            descricao: "payload complexo",
        },
        dados: [
            { id: 1, valor: 10 },
            { id: 2, valor: 20 },
            null,
        ],
        ok: true,
    });

    match complex {
        Ok(Value::Object(map)) => {
            // meta
            match map.get("meta") {
                Some(Value::Object(meta)) => {
                    assert_eq!(meta.get("versao"), Some(&Value::from(1)));
                    assert_eq!(
                        meta.get("descricao"),
                        Some(&Value::from("payload complexo"))
                    );
                }
                _ => panic!("campo 'meta' inválido"),
            }

            // dados
            match map.get("dados") {
                Some(Value::Array(dados)) => {
                    assert_eq!(dados.len(), 3);

                    match &dados[0] {
                        Value::Object(obj) => {
                            assert_eq!(obj.get("id"), Some(&Value::from(1)));
                            assert_eq!(obj.get("valor"), Some(&Value::from(10)));
                        }
                        _ => panic!("dados[0] inválido"),
                    }

                    match &dados[1] {
                        Value::Object(obj) => {
                            assert_eq!(obj.get("id"), Some(&Value::from(2)));
                            assert_eq!(obj.get("valor"), Some(&Value::from(20)));
                        }
                        _ => panic!("dados[1] inválido"),
                    }

                    assert!(matches!(dados[2], Value::Null));
                }
                _ => panic!("campo 'dados' inválido"),
            }

            assert_eq!(map.get("ok"), Some(&Value::from(true)));
        }
        Ok(_) => panic!("complex não é objeto"),
        Err(e) => panic!("json_safe! complexo retornou erro: {e}"),
    }

    println!("Todos os testes de json_safe! em main passaram");
}
