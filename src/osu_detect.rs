use winreg::enums::{ HKEY_CLASSES_ROOT, KEY_READ};
use winreg::RegKey;

pub fn where_is_osu() -> Result<String, ()>
{
    //NOTE: Basically we're fetching the path that Osu! uses to handle .osz/.osu and etc. files.
    let classes_root = winreg::RegKey::predef(HKEY_CLASSES_ROOT);
    let osu_reg_key = classes_root.open_subkey_with_flags(r#"osu\shell\open\command"#, KEY_READ);
    let registry_key: RegKey;

    match osu_reg_key {
        Ok(v) => { registry_key = v },
        Err(_) => { return Err(()); }
    }

    let value: String = registry_key.get_value("").unwrap();
    let value_parts: Vec<&str> = value.split(" ").collect();

    if value_parts.len() >= 1 
    {
        let path_string = value_parts[0].replace("\"", "");
        let path_without_file = path_string.replace(r#"\osu!.exe"#, "");
        Ok(path_without_file)
    }
    else
    {
        Err(())
    }
}