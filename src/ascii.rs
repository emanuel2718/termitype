pub struct AsciiArt {
    pub name: &'static str,
    pub art: &'static str,
}

pub const DEFAULT_ASCII_ART_NAME: &str = "Termitype";

/// Returns OS dependent ASCII art.
pub fn get_os_default_ascii_art() -> &'static str {
    if cfg!(target_os = "macos") {
        "Apple"
    } else if cfg!(target_os = "windows") {
        "Windows7"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else {
        DEFAULT_ASCII_ART_NAME
    }
}

pub fn get_ascii_art_by_name(name: &str) -> Option<&'static str> {
    AVAILABLE_ASCII_ARTS
        .iter()
        .find(|art| art.name == name)
        .map(|art| art.art)
}

pub fn available_ascii_arts() -> Vec<String> {
    AVAILABLE_ASCII_ARTS
        .iter()
        .map(|a| a.name.to_string())
        .collect()
}

pub fn print_ascii_list() {
    let mut arts: Vec<String> = available_ascii_arts();
    arts.sort_by_key(|k| k.to_lowercase());

    println!("\n• Available Ascii Arts ({}):", arts.len());

    println!("{}", "─".repeat(40));

    for ascii in arts {
        let is_default = ascii == DEFAULT_ASCII_ART_NAME;
        let name = if is_default {
            format!("{ascii} (default)")
        } else {
            ascii
        };
        println!("  • {name}");
    }
}

// some are courtesy of https://www.twitchquotes.com/copypastas/ascii-art
// others are manually converted using: https://github.com/TheZoraiz/ascii-image-converter
// others are taken directly from neofetch with `neofetch --ascii-distro <distro>`
// others are taken from https://steamcommunity.com/sharedfiles/filedetails/?id=3030708568
// others are taken from the great https://oldcompcz.github.io/jgs/joan_stark/
pub const AVAILABLE_ASCII_ARTS: &[AsciiArt] = &[
    AsciiArt {
        name: "Android",
        art: r#"
⣿⣿⣿⣿⣿⣿⣧⠻⣿⣿⠿⠿⠿⢿⣿⠟⣼⣿⣿⣿⣿⣿⣿
⣿⣿⣿⣿⣿⣿⠟⠃⠁⠀⠀⠀⠀⠀⠀⠘⠻⣿⣿⣿⣿⣿⣿
⣿⣿⣿⣿⡿⠃⠀⣴⡄⠀⠀⠀⠀⠀⣴⡆⠀⠘⢿⣿⣿⣿⣿
⣿⣿⣿⣿⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠘⣿⣿⣿⣿
⣿⠿⢿⣿⠶⠶⠶⠶⠶⠶⠶⠶⠶⠶⠶⠶⠶⠶⠶⣿⡿⠿⣿
⡇⠀⠀⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⠀⠀⢸
⡇⠀⠀⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⠀⠀⢸
⡇⠀⠀⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⠀⠀⢸
⡇⠀⠀⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⠀⠀⢸
⣧⣤⣤⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣤⣤⣼
⣿⣿⣿⣿⣶⣤⡄⠀⠀⠀⣤⣤⣤⠀⠀⠀⢠⣤⣴⣿⣿⣿⣿
⣿⣿⣿⣿⣿⣿⡇⠀⠀⠀⣿⣿⣿⠀⠀⠀⢸⣿⣿⣿⣿⣿⣿
⣿⣿⣿⣿⣿⣿⣿⣤⣤⣴⣿⣿⣿⣦⣤⣤⣾⣿⣿⣿⣿
        "#,
    },
    AsciiArt {
        name: "Anime Girl",
        art: r#"
 ⣇⣿⠘⣿⣿⣿⡿⡿⣟⣟⢟⢟⢝⠵⡝⣿⡿⢂⣼⣿⣷⣌⠩⡫⡻⣝⠹⢿⣿⣷
 ⡆⣿⣆⠱⣝⡵⣝⢅⠙⣿⢕⢕⢕⢕⢝⣥⢒⠅⣿⣿⣿⡿⣳⣌⠪⡪⣡⢑⢝⣇
 ⡆⣿⣿⣦⠹⣳⣳⣕⢅⠈⢗⢕⢕⢕⢕⢕⢈⢆⠟⠋⠉⠁⠉⠉⠁⠈⠼⢐⢕⢽
 ⡗⢰⣶⣶⣦⣝⢝⢕⢕⠅⡆⢕⢕⢕⢕⢕⣴⠏⣠⡶⠛⡉⡉⡛⢶⣦⡀⠐⣕⢕
 ⡝⡄⢻⢟⣿⣿⣷⣕⣕⣅⣿⣔⣕⣵⣵⣿⣿⢠⣿⢠⣮⡈⣌⠨⠅⠹⣷⡀⢱⢕
 ⡝⡵⠟⠈⢀⣀⣀⡀⠉⢿⣿⣿⣿⣿⣿⣿⣿⣼⣿⢈⡋⠴⢿⡟⣡⡇⣿⡇⡀⢕
 ⡝⠁⣠⣾⠟⡉⡉⡉⠻⣦⣻⣿⣿⣿⣿⣿⣿⣿⣿⣧⠸⣿⣦⣥⣿⡇⡿⣰⢗⢄
 ⠁⢰⣿⡏⣴⣌⠈⣌⠡⠈⢻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣬⣉⣉⣁⣄⢖⢕⢕⢕
 ⡀⢻⣿⡇⢙⠁⠴⢿⡟⣡⡆⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣵⣵⣿
 ⡻⣄⣻⣿⣌⠘⢿⣷⣥⣿⠇⣿⣿⣿⣿⣿⣿⠛⠻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
 ⣷⢄⠻⣿⣟⠿⠦⠍⠉⣡⣾⣿⣿⣿⣿⣿⣿⢸⣿⣦⠙⣿⣿⣿⣿⣿⣿⣿⣿⠟
 ⡕⡑⣑⣈⣻⢗⢟⢞⢝⣻⣿⣿⣿⣿⣿⣿⣿⠸⣿⠿⠃⣿⣿⣿⣿⣿⣿⡿⠁⣠
 ⡝⡵⡈⢟⢕⢕⢕⢕⣵⣿⣿⣿⣿⣿⣿⣿⣿⣿⣶⣶⣿⣿⣿⣿⣿⠿⠋⣀⣈⠙
 ⡝⡵⡕⡀⠑⠳⠿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠿⠛⢉⡠⡲⡫⡪⡪⡣
        "#,
    },
    AsciiArt {
        name: "Anime Tounge",
        art: r#"
⣾⣿⠿⠿⠶⠿⢿⣿⣿⣿⣿⣦⣤⣄⢀⡅⢠⣾⣛⡉⠄⠄⠄⠸⢀⣿⠄
⢀⡋⣡⣴⣶⣶⡀⠄⠄⠙⢿⣿⣿⣿⣿⣿⣴⣿⣿⣿⢃⣤⣄⣀⣥⣿⣿⠄
⢸⣇⠻⣿⣿⣿⣧⣀⢀⣠⡌⢻⣿⣿⣿⣿⣿⣿⣿⣿⣿⠿⠿⠿⣿⣿⣿⠄
⢸⣿⣷⣤⣤⣤⣬⣙⣛⢿⣿⣿⣿⣿⣿⣿⡿⣿⣿⡍⠄⠄⢀⣤⣄⠉⠋⣰
⣖⣿⣿⣿⣿⣿⣿⣿⣿⣿⢿⣿⣿⣿⣿⣿⢇⣿⣿⡷⠶⠶⢿⣿⣿⠇⢀⣤
⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣽⣿⣿⣿⡇⣿⣿⣿⣿⣿⣿⣷⣶⣥⣴⣿⡗
⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡟⠄
⣦⣌⣛⣻⣿⣿⣧⠙⠛⠛⡭⠅⠒⠦⠭⣭⡻⣿⣿⣿⣿⣿⣿⣿⣿⡿⠃⠄
⣿⣿⣿⣿⣿⣿⣿⡆⠄⠄⠄⠄⠄⠄⠄⠄⠹⠈⢋⣽⣿⣿⣿⣿⣵⣾⠃⠄
⣿⣿⣿⣿⣿⣿⣿⣿⠄⣴⣿⣶⣄⠄⣴⣶⠄⢀⣾⣿⣿⣿⣿⣿⣿⠃⠄⠄
⠈⠻⣿⣿⣿⣿⣿⣿⡄⢻⣿⣿⣿⠄⣿⣿⡀⣾⣿⣿⣿⣿⣛⠛⠁⠄⠄⠄
⠄⠄⠈⠛⢿⣿⣿⣿⠁⠞⢿⣿⣿⡄⢿⣿⡇⣸⣿⣿⠿⠛⠁⠄⠄⠄⠄⠄
⠄⠄⠄⠄⠄⠉⠻⣿⣿⣾⣦⡙⠻⣷⣾⣿⠃⠿⠋⠁⠄⠄⠄⠄⠄⢀⣠⣴
⣿⣶⣶⣮⣥⣒⠲⢮⣝⡿⣿⣿⡆⣿⡿⠃⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄
        "#,
    },
    AsciiArt {
        name: "Apple",
        art: r#"
                    'c.
                 ,xNMM.
               .OMMMMo
               OMMM0,
     .;loddo:' loolloddol;.
   cKMMMMMMMMMMNWMMMMMMMMMM0:
 .KMMMMMMMMMMMMMMMMMMMMMMMWd.
 XMMMMMMMMMMMMMMMMMMMMMMMX.
;MMMMMMMMMMMMMMMMMMMMMMMM:
:MMMMMMMMMMMMMMMMMMMMMMMM:
.MMMMMMMMMMMMMMMMMMMMMMMMX.
 kMMMMMMMMMMMMMMMMMMMMMMMMWd.
 .XMMMMMMMMMMMMMMMMMMMMMMMMMMk
  .XMMMMMMMMMMMMMMMMMMMMMMMMK.
    kMMMMMMMMMMMMMMMMMMMMMMd
     ;KMMMMMMMWXXWMMMMMMMk.
       .cooc,.    .,coo:.
      "#,
    },
    AsciiArt {
        name: "Arch Linux",
        art: r#"
                 =-
                ====
              .======
              =+======.
            :-:-==+===+:
           -+++===++++++-
         .=++++++++=====+=
        :+++===============.
       :=======:.  .-=======:
      -=======.      :=======-
    .========-        ======--:
   :=========-        -=======-:
 .====-::.               ..:--===-
:-:.                           .:--.
        "#,
    },
    AsciiArt {
        name: "Battlestation",
        art: r#"
     ______________      _______
    |.------------.|    | ___  o|
    || >_         ||    |[_____]|
    ||            ||    |[_____]|
    ||            ||    |[====o]|
    ||            ||    |[_.--_]|
    ||____________||    |[_____]|
.==.|""  ......    |.==.|      :|
|::| '-.________.-' |::||      :|
|''|  (__________)-.|''||______:|
`""`_.............._\""`______
   /:::::::::::'':::\`;'-.-.  `\
  /::=========.:.-::"\ \ \--\   \
  \`""""""""""""""""`/  \ \__)   \
   `""""""""""""""""`    '========'
        "#,
    },
    AsciiArt {
        name: "Debian",
        art: r#"
              .
         .::::::::::::::.
      .:::::.         .:::::.
    .:::.                .::::
   ::.                     ::.
  ::.         ..           .::
 .:.         :              ::
 .:         .:             .:
 .:          :.           ::
  ::          ....    ....
  .::.             .
   .::.
     ::.
       .:.
         ....
        "#,
    },
    AsciiArt {
        name: "FeelsDankMan",
        art: r#"
⠄⠄⠄⠄⠄⠄⠄⠄⠄⣾⣿⣿⣿⣿⡄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄
⠄⠄⠄⠄⠄⠄⠄⠄⣼⣿⣿⣿⣿⣿⣿⣧⠄⠄⠄⠄⠄⣀⣀⠄⠄⠄⠄⠄⠄⠄
⠄⠄⠄⠄⠄⠄⢀⣾⣿⣿⣿⣿⣿⣿⣿⣿⣧⠄⠄⠄⣾⠛⠛⣷⢀⣾⠟⠻⣦⠄
⠄⠄⠄⠄⠄⠄⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⡀⠄⠄⢰⡿⠋⠄⠄⣠⡾⠋⠄
⠄⠄⠄⠄⠄⣰⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⡄⠄⣬⡄⠄⠄⠄⣭⡅⠄⠄
⠄⠄⠄⠄⢰⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡆⠄⠄⠄⠄⠄⠄⠄⠄⠄
⠄⠄⠄⠄⢛⣛⣛⣛⣛⣛⣛⣛⣛⡛⢋⣉⣭⣭⣥⣬⣤⣤⣀⠄⠄⠄⠄⠄⠄⠄
⠄⠄⣴⣵⣿⣟⡉⣥⣶⣶⠶⠶⠬⣉⡂⠹⣟⡫⠽⠟⢒⣒⠒⠆⠄⠄⠄⠄⠄⠄
⠄⣼⣿⣿⣿⣿⣿⣶⣭⣃⡈⠄⠄⠘⠃⡰⢶⣶⣿⠏⠄⠄⠙⡛⠄⠄⠄⠄⠄⠄
⢰⣿⣿⣿⣿⣿⣿⣿⣯⣉⣉⣩⣭⣶⣿⡿⠶⠶⠶⠶⠶⠾⣋⠄⠄⠄⠄⠄⠄⠄
⢾⣿⣿⣿⣿⣿⣿⣿⢩⣶⣒⠒⠶⢖⣒⣚⡛⠭⠭⠭⠍⠉⠁⠄⠄⠄⣀⣀⡀⠄
⠘⢿⣿⣿⣿⣿⣿⣿⣧⣬⣭⣭⣭⣤⡤⠤⠶⠟⣋⣀⣀⡀⢀⣤⣾⠟⠋⠈⢳⠄
⣴⣦⡒⠬⠭⣭⣭⣭⣙⣛⠋⠭⡍⠁⠈⠙⠛⠛⠛⠛⢻⠛⠉⢻⠁⠄⠄⠄⢸⡀
⣿⣿⣿⣿⣷⣦⣤⠤⢬⢍⣼⣦⡾⠛⠄⠄⠄⠄⠄⠄⠈⡇⠄⢸⠄⠄⠄⢦⣄⣇
⣿⣿⡿⣋⣭⣭⣶⣿⣶⣿⣿⣿⠟⠛⠃⠄⠄⠄⠄⠄⢠⠃⠄⡜⠄⠄⠄⠔⣿⣿
        "#,
    },
    AsciiArt {
        name: "FeelsGoodMan",
        art: r#"
⣿⣿⣿⣿⣿⣿⣿⡿⠛⠉⠉⠉⠉⠛⠻⣿⣿⠿⠛⠛⠙⠛⠻⣿⣿⣿⣿⣿⣿⣿
⣿⣿⣿⣿⣿⠟⠁⠀⠀⠀⢀⣀⣀⡀⠀⠈⢄⠀⠀⠀⠀⠀⠀⠀⢻⣿⣿⣿⣿⣿
⣿⣿⣿⣿⠏⠀⠀⠀⠔⠉⠁⠀⠀⠈⠉⠓⢼⡤⠔⠒⠀⠐⠒⠢⠌⠿⢿⣿⣿⣿
⣿⣿⣿⡏⠀⠀⠀⠀⠀⠀⢀⠤⣒⠶⠤⠭⠭⢝⡢⣄⢤⣄⣒⡶⠶⣶⣢⡝⢿⣿
⡿⠋⠁⠀⠀⠀⠀⣀⠲⠮⢕⣽⠖⢩⠉⠙⣷⣶⣮⡍⢉⣴⠆⣭⢉⠑⣶⣮⣅⢻
⠀⠀⠀⠀⠀⠀⠀⠉⠒⠒⠻⣿⣄⠤⠘⢃⣿⣿⡿⠫⣿⣿⣄⠤⠘⢃⣿⣿⠿⣿
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠓⠤⠭⣥⣀⣉⡩⡥⠴⠃⠀⠈⠉⠁⠈⠉⠁⣴⣾⣿
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⠤⠔⠊⠀⠀⠀⠓⠲⡤⠤⠖⠐⢿⣿⣿⣿
⠀⠀⠀⠀⠀⠀⠀⠀⣠⣄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢻⣿⣿
⠀⠀⠀⠀⠀⠀⠀⢸⣿⡻⢷⣤⣀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣘⣿⣿
⠀⠀⠀⠀⠀⠠⡀⠀⠙⢿⣷⣽⣽⣛⣟⣻⠷⠶⢶⣦⣤⣤⣤⣤⣶⠾⠟⣯⣿⣿
⠀⠀⠀⠀⠀⠀⠉⠂⠀⠀⠀⠈⠉⠙⠛⠻⠿⠿⠿⠿⠶⠶⠶⠶⠾⣿⣟⣿⣿⣿
⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣴⣿⣿⣿⣿⣿⣿
⣿⣿⣶⣤⣀⣀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣀⣤⣟⢿⣿⣿⣿⣿⣿⣿⣿
⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣶⣶⣶⣶⣶⣶⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
        "#,
    },
    AsciiArt {
        name: "Fedora",
        art: r#"
       ...::::::::......
      .::::::::::.:-=+****=::..
    .::::::::::.:*%@@@@@@@@+--::.
  .:::::::::::.-@@@@*--::::-----::.
 .:::::::::::::%@@@=.:::::::----:::.
.:::::::::.....%@@@:.:..::::----::::
::::::::::=====@@@@+====-:-----:::::.
:::::----*@@@@@@@@@@@@@@#----:::::::.
:::-----::-====@@@@+====::::::::::::
::----:::::..:.%@@@-....:::::::::::.
::----:::::::.-@@@@-::::::::::::::.
:::-------:--+@@@@+.::::::::::::.
::::--=@@@@@@@@%*-.::::::::::..
.::::::=+***+=-:.::::::::..
      "#,
    },
    AsciiArt {
        name: "Gentoo",
        art: r#"
          ....
      .-+*%%@@@%%#*=-.
    -*%@@@@@@@@@@@@%%#*+-.
  -%@@@@@@@@@@@@@%#%%%%##**=:
 +%@@@@@@@@@@@%***++-#%####*#*=.
:+#%@@@@@@@@@%*=::==-#%###****#%*.
 :=+*#%@@@@@@@@@%##%%%####****++@@:
   .:-=+*%@@@@@@@@@%%%####*****%@%=
      .+%@@@@@@@@%%%%####***#%@%*=.
    =#@@@@@@@@@@%%%%###**#%@@#*-.
 .+@@@@@@@@@@@@%%%####%%@%#*=:
-@@@@@@@@@@@%%%%%%%%@@%*+-:
*@@@@@@%%%%%%%%@@%#*+=:.
-*%@@@@@@@@%%#*+=-:.
 .-=+++++==--:..
          .
      "#,
    },
    AsciiArt {
        name: "Half Life",
        art: r#"
⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣀⣠⣤⣤⣴⣦⣤⣤⣄⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⢀⣤⣾⣿⣿⣿⣿⠿⠿⠿⠿⣿⣿⣿⣿⣶⣤⡀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⣠⣾⣿⣿⡿⠛⠉⠀⠀⠀⠀⠀⠀⠀⠀⠉⠛⢿⣿⣿⣶⡀⠀⠀⠀⠀
⠀⠀⠀⣴⣿⣿⠟⠁⠀⠀⠀⣶⣶⣶⣶⡆⠀⠀⠀⠀⠀⠀⠈⠻⣿⣿⣦⠀⠀⠀
⠀⠀⣼⣿⣿⠋⠀⠀⠀⠀⠀⠛⠛⢻⣿⣿⡀⠀⠀⠀⠀⠀⠀⠀⠙⣿⣿⣧⠀⠀
⠀⢸⣿⣿⠃⠀⠀⠀⠀⠀⠀⠀⠀⢀⣿⣿⣷⠀⠀⠀⠀⠀⠀⠀⠀⠸⣿⣿⡇⠀
⠀⣿⣿⡿⠀⠀⠀⠀⠀⠀⠀⠀⢀⣾⣿⣿⣿⣇⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⠀
⠀⣿⣿⡇⠀⠀⠀⠀⠀⠀⠀⢠⣿⣿⡟⢹⣿⣿⡆⠀⠀⠀⠀⠀⠀⠀⣹⣿⣿⠀
⠀⣿⣿⣷⠀⠀⠀⠀⠀⠀⣰⣿⣿⠏⠀⠀⢻⣿⣿⡄⠀⠀⠀⠀⠀⠀⣿⣿⡿⠀
⠀⢸⣿⣿⡆⠀⠀⠀⠀⣴⣿⡿⠃⠀⠀⠀⠈⢿⣿⣷⣤⣤⡆⠀⠀⣰⣿⣿⠇⠀
⠀⠀⢻⣿⣿⣄⠀⠀⠾⠿⠿⠁⠀⠀⠀⠀⠀⠘⣿⣿⡿⠿⠛⠀⣰⣿⣿⡟⠀⠀
⠀⠀⠀⠻⣿⣿⣧⣄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⣾⣿⣿⠏⠀⠀⠀
⠀⠀⠀⠀⠈⠻⣿⣿⣷⣤⣄⡀⠀⠀⠀⠀⠀⠀⢀⣠⣴⣾⣿⣿⠟⠁⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠈⠛⠿⣿⣿⣿⣿⣿⣶⣶⣿⣿⣿⣿⣿⠿⠋⠁⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠉⠛⠛⠛⠛⠛⠛⠉⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
        "#,
    },
    AsciiArt {
        name: "Linux",
        art: r#"
         _nnnn_
        dGGGGMMb
       @p~qp~~qMb
       M|@||@) M|
       @,----.JM|
      JS^\__/  qKL
     dZP        qKRb
    dZP          qKKb
   fZP            SMMb
   HZM            MMMM
   FqM            MMMM
 __| ".        |\dS"qML
 |    `.       | `' \Zq
_)      \.___.,|     .'
\____   )MMMMMM|   .'
     `-'       `--'
        "#,
    },
    AsciiArt {
        name: "Macintosh",
        art: r#"
        ______________
       /             /|
      /             / |
     /____________ /  |
    | ___________ |   |
    || >_        ||   |
    ||           ||   |
    ||           ||   |
    ||___________||   |
    |   _______   |  /
   /|  (_______)  | /
  ( |_____________|/
   \
.=======================.
| ::::::::::::::::  ::: |
| ::::::::::::::[]  ::: |
|   -----------     ::: |
`-----------------------'
        "#,
    },
    AsciiArt {
        name: "MonkaHmm",
        art: r#"
⠄⠄⠄⠄⠄⠄⢀⣠⣤⣶⣶⣶⣤⣄⠄⠄⢀⣠⣤⣤⣤⣤⣀⠄⠄⠄⠄⠄⠄⠄
⠄⠄⠄⠄⢠⣾⣿⣿⣿⣿⠿⠿⢿⣿⣿⡆⣿⣿⣿⣿⣿⣿⣿⣷⡄⠄⠄⠄⠄⠄
⠄⠄⠄⣴⣿⣿⡟⣩⣵⣶⣾⣿⣷⣶⣮⣅⢛⣫⣭⣭⣭⣭⣭⣭⣛⣂⠄⠄⠄⠄
⠄⠄⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣶⣭⠛⣿⣿⣿⣿⣿⣿⣿⣿⣦⡀⠄
⣠⡄⣿⣿⣿⣿⣿⣿⣿⠿⢟⣛⣫⣭⠉⠍⠉⣛⠿⡘⣿⠿⢟⣛⡛⠉⠙⠻⢿⡄
⣿⣿⣿⣿⣿⣿⣿⣿⣿⣶⣶⣶⣶⣶⣶⣶⣶⣭⣍⠄⣡⣬⣭⣭⣅⣈⣀⣉⣁⠄
⣿⣿⣿⣿⣿⣿⣿⣿⣶⣭⣛⡻⠿⠿⢿⣿⡿⢛⣥⣾⣿⣿⣿⣿⣿⣿⣿⠿⠋⠄
⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠿⣩⣵⣾⣿⣿⣯⣙⠟⣋⣉⣩⣍⡁⠄⠄⠄
⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣿⣿⣿⣿⣷⡄⠄⠄
⣿⣿⣿⣿⣿⣿⡿⢟⣛⣛⣛⣛⠿⠿⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠿⡀⠄
⣿⣿⣿⣿⣿⡟⢼⣿⣯⣭⣛⣛⣛⡻⠷⠶⢶⣬⣭⣭⣭⡭⠭⢉⡄⠶⠾⠟⠁⠄
⣿⣿⣿⣿⣟⠻⣦⣤⣭⣭⣭⣭⣛⣛⡻⠿⠷⠶⢶⣶⠞⣼⡟⡸⣸⡸⠿⠄⠄⠄
⣛⠿⢿⣿⣿⣿⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠷⡆⣾⠟⡴⣱⢏⡜⠆⠄⠄⠄
⣭⣙⡒⠦⠭⣭⣛⣛⣛⡻⠿⠿⠟⣛⣛⣛⣛⡋⣶⡜⣟⣸⣠⡿⣸⠇⣧⡀⠄⠄
⣿⣿⣿⣿⣷⣶⣦⣭⣭⣭⣭⣭⣭⣥⣶⣶⣶⡆⣿⣾⣿⣿⣿⣷⣿⣸⠉⣷⠄⠄
        "#,
    },
    AsciiArt {
        name: "MonkaS",
        art: r#"
⣿⣿⣿⣿⣿⡿⠛⠋⠁⠀⠀⠀⠀⠙⠛⠿⠟⠋⠉⠁⠀⠈⠙⠻⣿⣿⣿⣿⣿⣿
⣿⣿⣿⡿⠋⠀⠸⠄⢀⣀⠠⠤⠤⣀⡀⠐⡄⠀⠀⠀⠀⠀⠾⠂⠈⠻⣿⣿⣿⣿
⣿⣿⡟⠀⠀⠀⠠⠋⠁⠀⠀⠀⠀⠀⠉⠙⠻⠒⠚⠛⠛⠛⠛⠒⠒⠦⠘⢿⣿⣿
⣿⠟⠀⠀⡆⠀⠀⠀⠀⢀⣤⣴⣶⣶⣶⣶⣶⣧⣄⢀⣠⣤⣤⣶⣶⣶⣤⣤⣙⢿
⠁⢸⠀⠸⠅⠀⠀⣴⣾⣿⣿⣿⣿⣿⡏⡉⠙⣿⣿⣼⣿⣿⣿⣿⣿⢋⡉⢻⣿⡆
⣀⠀⠀⠀⠀⠀⠀⠀⢉⠛⠿⢿⣿⣿⣷⣧⣶⡿⠟⠸⠿⠿⣿⡿⠿⠶⠬⠾⠿⢃
⠓⠀⠀⠀⠀⠀⠀⠀⠈⠙⠒⠤⠤⠬⠭⠁⣤⠤⠖⠁⠀⠀⠀⠀⠀⠀⠀⠀⣠⣾
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠠⠔⠊⠁⠀⠀⠀⠑⠢⡄⠒⠒⠂⠰⣿⣿⣿
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠘⢿⣿
⠀⠀⠀⠀⠀⠀⠀⣀⣤⣤⣤⣀⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⡈⣿
⠀⠀⠀⠀⠀⠀⣼⣿⣧⣭⣭⣍⣛⣛⣛⡶⠶⠶⢦⣤⣤⣤⣤⣤⣴⣶⠿⠛⣡⣿
⠀⠀⠀⠀⠰⢄⠈⠉⠉⠉⠉⠉⠉⠙⠛⠛⠿⠿⠿⠷⠶⠶⣶⣶⣶⡶⢟⣸⣿⣿
⣄⣀⡀⠀⠀⠀⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⣴⣿⣿⣿⣿⣿
⠀⠀⠈⠉⠓⠒⠢⠤⠤⠤⠤⣤⣤⣤⣤⠤⠄⠀⠀⠀⢴⣶⣿⣿⣿⣿⣿⣿⣿⣿
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⢻⣿⣿⣿⣿⣿⣿⣿
        "#,
    },
    AsciiArt {
        name: "Nier Automata 2B",
        art: r#"
⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡽⣝⡾⡽⣮⢳⣝⢮⣳⣝⢽⡪⣏⢮⡳⡱⡕⢕⢅⢍⢛⢿⢿⣿⣿
⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣳⡿⣿⡿⣗⣟⣯⡿⣷⣝⣷⡳⡽⣧⡫⣎⢧⡫⡧⡫⡪⡪⡢⡑⡐⡙⢽⣿
⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣻⠮⡻⣿⢿⣿⣷⣻⣽⢿⣾⣞⣿⡽⡵⣟⢮⡪⣺⢸⢵⡱⡱⡘⡜⡔⡌⡂⣿
⣿⣿⣿⣿⣿⣟⣯⣿⣿⣳⢯⠎⡇⠈⢻⢯⡻⣿⣟⣾⡯⡷⣿⣞⣿⡯⡯⣗⣝⢜⢵⠱⡧⡣⡣⡂⢑⠱⡂⣿
⣿⣿⣿⣷⢿⡿⡽⣟⣗⡯⡊⠀⠢⠀⠀⠙⠝⢎⢟⢽⣟⣯⡻⣾⡺⣿⡹⣪⢮⢪⡪⡣⡣⡃⡪⡪⡐⡐⡨⣶
⣿⣟⡷⡿⡽⣽⣻⣟⣞⢕⠀⠀⠀⠡⠀⠀⠈⢀⠡⠀⠑⠯⡫⣞⣝⢷⢝⡎⢎⡎⡮⡺⡸⡢⡊⢎⠢⡂⡂⣻
⣗⡳⣝⢽⢪⢺⣞⣞⢮⠀⠀⠀⠂⠀⠁⣠⣺⣯⡯⡹⡸⡰⡑⡌⢮⢫⢞⠎⡨⢪⡻⢜⠜⢌⠪⡊⠇⢌⠢⡹
⡪⡺⡪⠨⡊⡎⣟⡮⡃⠡⠀⠀⢠⣰⣽⣿⣟⢍⠡⡁⢱⢑⢕⢜⢘⠎⠭⠡⠈⠐⠅⠈⡇⡑⠌⠌⢌⠢⢑⣼
⡪⡪⡪⠌⢆⢣⠣⢯⠱⠀⠐⣜⢽⡽⣾⢯⣿⠳⠑⠘⠐⠑⠜⡰⠡⠣⡑⠡⠁⠀⠅⢐⢈⠢⠡⢑⠠⢁⣎⣿
⣇⡊⡺⡌⡘⢜⢌⠪⠐⠀⠈⢎⠳⡹⡝⣝⢦⣑⠢⠁⠂⠄⠁⠪⢈⠂⠂⠁⠀⠠⠁⡐⠠⠡⢈⠐⣈⡾⣗⣿
⣷⡱⡀⠣⡈⡂⡂⠅⠂⡈⠀⠀⠈⠂⠕⡑⡗⣗⡯⣗⡕⡅⠐⠀⠐⠈⠀⠠⠈⠐⡠⢈⠨⡠⣂⢦⣷⣿⣿⣿
⣿⣿⣮⣂⢂⠐⠠⢁⢱⡄⢌⠀⠄⠨⠠⡀⡈⠂⠙⠘⠘⠈⢀⠐⢀⠀⠘⠧⣌⣞⣠⣶⣽⣿⣿⣿⣿⣿⣿⣿
⣿⣿⣿⣿⣷⣮⣔⢤⣸⣷⣿⡷⡄⠀⠀⠀⠈⠈⠀⠁⠈⠈⠀⠀⠀⠀⠀⠀⠀⠉⢿⣿⣿⡿⣿⣿⣿⣿⣿⣿
⠛⠛⢿⣿⣿⣿⣿⣿⣿⡿⠿⠛⠁⠀⠀⠀⠀⠀⢀⠀⠂⠁⠁⠀⠂⠀⠀⠀⠀⠀⠀⠻⠿⣿⡿⠟⠙⠙⠻⢿
⠀⠀⠀⠈⠉⠉⠁⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠀⠀⠀⠐⠀⠀⠀⠀⠀⠀⠀⠀
        "#,
    },
    AsciiArt {
        name: "Nier Automata 2B (Alt)",
        art: r#"
⣿⢿⣿⣻⣿⣿⢿⣿⣻⣿⡟⣼⣿⣯⢳⠱⡕⢌⢂⠪⢐⢐⢑⠐⠅⡂⡂⠔⠀⡍⢯⣟⣿⣽⣻⡽⡯⣯⣳⣝
⣿⣻⢷⡿⣷⣿⣻⣽⡷⣿⢪⣿⣿⣺⢱⠡⡫⡢⠢⡑⡐⠔⠄⠅⢅⢂⢂⠡⢁⠈⢢⢻⣳⣻⣺⢽⢯⣗⣟⣾
⣿⡽⣟⣿⣻⣾⢿⣽⢯⣳⣻⢿⣳⣻⠰⡁⢣⠪⡨⡂⢌⢌⠪⢈⠢⠐⡀⢂⠐⠄⠐⣧⣳⣳⢯⢯⣻⣺⣺⣽
⣷⣟⣿⡽⣿⢾⡿⣯⢧⡳⣹⢽⡳⡣⡑⠠⠐⡑⡐⢌⢂⠢⡑⡐⠨⠐⢐⠠⠀⠡⠨⣞⣞⢮⢯⣳⣳⣳⣻⣞
⣷⣻⣞⣿⣻⣟⣿⣟⢎⢎⢪⠣⡣⠣⠨⠀⠀⠐⢈⢂⠢⡑⡐⠄⠡⠡⠐⢀⠁⠐⢐⡜⡮⡯⣗⣗⣗⢗⣗⡯
⣷⣻⣞⣷⣻⣺⣟⡎⠆⢕⢐⠑⢌⢊⠂⠀⠀⠀⠀⠀⠅⢂⠂⢅⠨⢀⠡⠀⠄⠁⡀⣷⡹⣽⣳⣳⡳⣝⢮⢯
⣗⣗⣗⣗⣗⣏⣾⢑⠅⠅⡂⠨⢐⠐⠀⠀⠀⢀⢌⢐⠠⠐⠨⠐⠀⢐⠀⠂⠐⠀⡀⢷⢯⣗⣗⢷⢝⢮⠯⡮
⣗⢯⣺⣺⡺⣼⡺⠐⠨⠐⠠⢈⠐⡈⠀⠀⡐⡐⡈⠀⠠⠁⢂⠁⡈⠀⠀⡁⠄⠂⢰⡜⣗⡗⣗⢝⢵⡹⣕⢝
⣯⣳⡳⡵⣝⢾⠌⡌⠄⡁⠂⡀⢂⠀⠀⡐⢐⠐⠄⠁⠈⠐⠠⠀⡀⠀⠄⠀⠐⠀⡌⣟⢮⢯⢺⢸⡱⣕⢕⣕
⣗⣗⣝⢞⢮⢳⡑⡕⡀⠂⠄⠠⠀⠀⠀⢐⠠⢁⠂⠐⠀⠁⠐⠀⠀⠀⠄⠈⡀⢰⡣⡳⡹⡜⡕⣕⢵⢱⡱⣕
⣗⣗⢵⢝⢕⢕⢕⢅⢕⡐⠀⠄⠂⠈⠀⠀⠐⠀⠅⡑⡈⠀⠀⠀⠀⢐⠀⡡⡰⣝⢜⢎⢎⢎⢎⢎⢎⢇⢇⢮
⢧⡳⡝⣎⢇⢇⢇⢇⢆⢇⡂⠠⠀⠰⡀⠀⢀⠁⠀⠂⢀⠀⠠⠀⢈⠆⡴⡱⡕⣕⢝⢎⢇⢇⢇⢇⢇⢇⢇⢗
⡳⣕⢇⢇⠇⡇⡣⡱⡨⡢⠪⡢⠠⡑⡜⡔⠀⠈⠀⠈⠀⠀⠀⠀⠀⠙⣜⢮⢺⡸⡱⡱⡱⡱⡱⡑⡕⢅⠣⡣
⡣⡣⡑⢅⠣⡊⢌⢂⠪⠈⠪⡪⡪⡪⠊⠂⠀⠀⠀⠀⠀⠂⠀⠀⠀⠀⠸⠪⡇⠁⠑⢕⢕⢕⢅⠕⡌⡢⡑⢌
⠪⡂⡊⠢⡑⢌⠔⢈⠀⠄⠀⠁⠀⠀⠀⠀⠀⠀⠀⠐⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠢⡑⡑⢔⢐⢌⠢
⢑⢐⢈⠢⢨⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠌⡂⡂⡢⢑
        "#,
    },
    AsciiArt {
        name: "Nier Automata 2B (Small)",
        art: r#"
⠄⠄⠄⠄⢠⣿⣿⣿⣿⣿⢻⣿⣿⣿⣿⣿⣿⣿⣿⣯⢻⣿⣿⣿⣿⣆⠄⠄⠄
⠄⠄⣼⢀⣿⣿⣿⣿⣏⡏⠄⠹⣿⣿⣿⣿⣿⣿⣿⣿⣧⢻⣿⣿⣿⣿⡆⠄⠄
⠄⠄⡟⣼⣿⣿⣿⣿⣿⠄⠄⠄⠈⠻⣿⣿⣿⣿⣿⣿⣿⣇⢻⣿⣿⣿⣿⠄⠄
⠄⢰⠃⣿⣿⠿⣿⣿⣿⠄⠄⠄⠄⠄⠄⠙⠿⣿⣿⣿⣿⣿⠄⢿⣿⣿⣿⡄⠄
⠄⢸⢠⣿⣿⣧⡙⣿⣿⡆⠄⠄⠄⠄⠄⠄⠄⠈⠛⢿⣿⣿⡇⠸⣿⡿⣸⡇⠄
⠄⠈⡆⣿⣿⣿⣿⣦⡙⠳⠄⠄⠄⠄⠄⠄⢀⣠⣤⣀⣈⠙⠃⠄⠿⢇⣿⡇⠄
⠄⠄⡇⢿⣿⣿⣿⣿⡇⠄⠄⠄⠄⠄⣠⣶⣿⣿⣿⣿⣿⣿⣷⣆⡀⣼⣿⡇⠄
⠄⠄⢹⡘⣿⣿⣿⢿⣷⡀⠄⢀⣴⣾⣟⠉⠉⠉⠉⣽⣿⣿⣿⣿⠇⢹⣿⠃⠄
⠄⠄⠄⢷⡘⢿⣿⣎⢻⣷⠰⣿⣿⣿⣿⣦⣀⣀⣴⣿⣿⣿⠟⢫⡾⢸⡟⠄⠄
⠄⠄⠄⠄⠻⣦⡙⠿⣧⠙⢷⠙⠻⠿⢿⡿⠿⠿⠛⠋⠉⠄⠂⠘⠁⠞⠄⠄⠄
⠄⠄⠄⠄⠄⠈⠙⠑⣠⣤⣴⡖⠄⠿⣋⣉⣉⡁⠄⢾⣦⠄⠄⠄⠄⠄⠄⠄⠄
        "#,
    },
    AsciiArt {
        name: "Nix OS",
        art: r#"
⠀⠀⠀⠀⠀⠀⠀⠀⠀⡐⠐⡐⠠⠀⠀⠀⠀⠀⠑⡱⡱⡱⡱⠄⠀⠀⢀⢨⢪⢪⢢⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢁⠐⡈⡐⠀⠀⠀⠀⠀⠀⢕⢕⢕⢕⢄⢠⢪⢪⢪⢪⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠐⠀⡂⢐⠀⡀⠀⠀⠀⠀⠀⢑⢕⢕⢕⢕⢕⢕⠕⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⡀⠅⠡⠐⡈⠄⢂⠐⡐⠐⡐⢀⠂⠄⠡⠁⡂⠡⡣⡣⡣⡣⡃⠁⠀⠀⠀⠀⠀⢀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠄⠂⠨⠀⠅⠐⠈⠄⠂⠄⠡⠐⠠⠈⠌⠠⠁⠐⠠⠑⡱⡸⡸⡰⡀⠀⠀⠀⠀⡐⢀⠂⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡰⡱⡱⡱⡱⠁⠀⠀⠀⠀⠀⠀⠀⠀⠈⠸⡸⡸⡸⡀⠀⢈⠐⡀⢂⠂⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⡔⡕⡕⡕⠕⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⢸⢸⢸⠈⡀⢂⠂⢂⠂⠀⠀⠀⠀⠀
⢀⢆⢏⢕⢕⢕⢝⢜⢜⢜⢜⢜⠌⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠨⢂⠐⢐⠠⠈⠄⢂⠡⠈⠌⠠⢀
⠑⡕⡕⡕⡕⡕⡕⡕⡕⡕⡕⠁⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠠⠈⠄⢂⠡⠈⠄⢂⠡⢈⠐⠀
⠀⠀⠀⠀⠀⢐⢜⢜⢜⠬⠀⠄⠡⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠌⡈⠌⠐⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⢠⢪⢪⢪⠊⠀⠁⠌⠄⠅⠠⠀⠀⠀⠀⠀⠀⠀⠀⠀⠠⠈⠄⠡⠐⠈⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠃⢇⢇⠇⠀⠀⠀⠀⠈⠄⠡⢁⠐⠔⡤⢤⢰⢰⢰⢰⢰⢰⢰⢰⢰⢰⢰⢰⢰⢰⢰⠐⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠑⠁⠀⠀⠀⠀⠀⠈⠌⢐⢀⢂⠈⢘⢜⢜⢜⢜⢜⢜⢜⢜⢜⢜⢜⢜⢜⢜⢜⠂⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠠⠀⠌⡐⡀⢂⠐⡈⠄⡀⠀⠀⠀⠀⠘⡜⡜⡜⡔⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠠⢈⠨⠐⡀⠂⠐⡀⢂⠂⡐⠀⠀⠀⠀⠀⠐⠱⡱⡸⡱⢄⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠐⡀⢂⠁⠀⠀⠀⠐⠀⢂⠂⠂⠄⠀⠀⠀⠀⠈⢪⢪⢪⠊⠀⠀
        "#,
    },
    AsciiArt {
        name: "Pause Champ",
        art: r#"
⣿⣿⣿⣿⡿⠟⠛⠛⠛⠛⠉⠉⠙⠛⠛⠿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠿⠟⠂⠀
⣿⣿⣯⣤⣤⣶⣶⣶⣶⣿⣿⣿⣿⣿⣿⣶⣾⣿⣿⣿⣿⣿⣿⣿⣏⠀⣀⣀⡀⠀
⣿⣿⣿⣿⣿⣿⣿⡿⠿⠟⠛⠛⠿⠟⠉⠉⠉⢻⣿⣿⣿⡿⠟⠛⢉⣼⣿⣿⣿⠀
⣿⣿⣿⣿⣭⣤⣴⣶⣿⣿⠃⠀⠀⢀⣀⣤⣶⣿⣿⣿⣿⡇⠀⠀⣩⣤⣤⠀⠀⠀
⣿⣿⣿⣿⣿⣿⣟⠛⠛⠛⠛⢋⣩⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣟⠛⠛⠃⠀⠀⠀
⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣦⣤⣤⣤⣄⠀
⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡄
⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⢿⣿⡿⢿⣿⣿⣿⣿⣿⣿⠃
⠿⠿⠿⠿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣭⣁⣀⡀⠀⠀⠀⠀⠀⢠⣾⣿⣿⠏⠀
⠀⠀⠀⠀⠀⠀⠉⣻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣶⣶⣤⣈⡻⠋⠁⠀⠀
⣰⣶⣶⣶⣶⣶⣶⣿⣿⣿⣿⣿⣿⣿⡿⠿⠿⠿⠛⠛⠛⠛⠛⠛⠛⠩⠀⠀⠀⠀
⣿⣿⣿⣿⣿⣿⠉⠉⠉⣿⣿⡶⠶⠶⢶⠿⠿⠛⠛⠛⠛⠛⠛⣻⣿⠃⠀⠀⠀⠀
⠛⠛⣿⣿⣿⣿⣷⡀⠀⠈⠛⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣼⠋⠀⠀⠀⠀⠀⠀
⢠⣾⣿⣿⣿⣿⣿⠿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⡴⠋⠀⠀⠀⠀⠀⠀⠀
⠄⠙⠛⠿⣿⣿⣇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⠞⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀
        "#,
    },
    AsciiArt {
        name: "Pepe",
        art: r#"
⣿⣿⣿⣿⣿⣿⠿⢋⣥⣴⣶⣶⣶⣬⣙⠻⠟⣋⣭⣭⣭⣭⡙⠻⣿⣿⣿⣿⣿
⣿⣿⣿⣿⡿⢋⣴⣿⣿⠿⢟⣛⣛⣛⠿⢷⡹⣿⣿⣿⣿⣿⣿⣆⠹⣿⣿⣿⣿
⣿⣿⣿⡿⢁⣾⣿⣿⣴⣿⣿⣿⣿⠿⠿⠷⠥⠱⣶⣶⣶⣶⡶⠮⠤⣌⡙⢿⣿
⣿⡿⢛⡁⣾⣿⣿⣿⡿⢟⡫⢕⣪⡭⠥⢭⣭⣉⡂⣉⡒⣤⡭⡉⠩⣥⣰⠂⠹
⡟⢠⣿⣱⣿⣿⣿⣏⣛⢲⣾⣿⠃⠄⠐⠈⣿⣿⣿⣿⣿⣿⠄⠁⠃⢸⣿⣿⡧
⢠⣿⣿⣿⣿⣿⣿⣿⣿⣇⣊⠙⠳⠤⠤⠾⣟⠛⠍⣹⣛⣛⣢⣀⣠⣛⡯⢉⣰
⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⡶⠶⢒⣠⣼⣿⣿⣛⠻⠛⢛⣛⠉⣴⣿⣿
⣿⣿⣿⣿⣿⣿⣿⡿⢛⡛⢿⣿⣿⣶⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⡈⢿⣿
⣿⣿⣿⣿⣿⣿⣿⠸⣿⡻⢷⣍⣛⠻⠿⠿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠿⢇⡘⣿
⣿⣿⣿⣿⣿⣿⣿⣷⣝⠻⠶⣬⣍⣛⣛⠓⠶⠶⠶⠤⠬⠭⠤⠶⠶⠞⠛⣡⣿
⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣶⣬⣭⣍⣙⣛⣛⣛⠛⠛⠛⠿⠿⠿⠛⣠⣿⣿
⣦⣈⠉⢛⠻⠿⠿⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠿⠛⣁⣴⣾⣿⣿⣿⣿
⣿⣿⣿⣶⣮⣭⣁⣒⣒⣒⠂⠠⠬⠭⠭⠭⢀⣀⣠⣄⡘⠿⣿⣿⣿⣿⣿⣿⣿
⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣦⡈⢿⣿⣿⣿⣿⣿
        "#,
    },
    AsciiArt {
        name: "PepeHackerman",
        art: r#"
⠄⠄⠄⠄⠄⠄⠄⢀⣠⣶⣾⣿⣶⣦⣤⣀⠄⢀⣀⣤⣤⣤⣤⣄⠄⠄⠄⠄⠄⠄
⠄⠄⠄⠄⠄⢀⣴⣿⣿⣿⡿⠿⠿⠿⠿⢿⣷⡹⣿⣿⣿⣿⣿⣿⣷⠄⠄⠄⠄⠄
⠄⠄⠄⠄⠄⣾⣿⣿⣿⣯⣵⣾⣿⣿⡶⠦⠭⢁⠩⢭⣭⣵⣶⣶⡬⣄⣀⡀⠄⠄
⠄⠄⠄⡀⠘⠻⣿⣿⣿⣿⡿⠟⠩⠶⠚⠻⠟⠳⢶⣮⢫⣥⠶⠒⠒⠒⠒⠆⠐⠒
⠄⢠⣾⢇⣿⣿⣶⣦⢠⠰⡕⢤⠆⠄⠰⢠⢠⠄⠰⢠⠠⠄⡀⠄⢊⢯⠄⡅⠂⠄
⢠⣿⣿⣿⣿⣿⣿⣿⣏⠘⢼⠬⠆⠄⢘⠨⢐⠄⢘⠈⣼⡄⠄⠄⡢⡲⠄⠂⠠⠄
⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣥⣀⡁⠄⠘⠘⠘⢀⣠⣾⣿⢿⣦⣁⠙⠃⠄⠃⠐⣀
⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣋⣵⣾⣿⣿⣿⣿⣦⣀⣶⣾⣿⣿⡉⠉⠉
⣿⣿⣿⣿⣿⣿⣿⠟⣫⣥⣬⣭⣛⠿⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡆⠄
⣿⣿⣿⣿⣿⣿⣿⠸⣿⣏⣙⠿⣿⣿⣶⣦⣍⣙⠿⠿⠿⠿⠿⠿⠿⠿⣛⣩⣶⠄
⣛⣛⣛⠿⠿⣿⣿⣿⣮⣙⠿⢿⣶⣶⣭⣭⣛⣛⣛⣛⠛⠛⠻⣛⣛⣛⣛⣋⠁⢀
⣿⣿⣿⣿⣿⣶⣬⢙⡻⠿⠿⣷⣤⣝⣛⣛⣛⣛⣛⣛⣛⣛⣛⠛⠛⣛⣛⠛⣡⣴⣿
⣛⣛⠛⠛⠛⣛⡑⡿⢻⢻⠲⢆⢹⣿⣿⣿⣿⣿⣿⠿⠿⠟⡴⢻⢋⠻⣟⠈⠿⠿
⣿⡿⡿⣿⢷⢤⠄⡔⡘⣃⢃⢰⡦⡤⡤⢤⢤⢤⠒⠞⠳⢸⠃⡆⢸⠄⠟⠸⠛⢿
⡟⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠁⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⢸
"#,
    },
    AsciiArt {
        name: "PepeJAM",
        art: r#"
⠄⠄⠄⠄⠄⠄⠄⣠⣴⣶⣿⣿⡿⠶⠄⠄⠄⠄⠐⠒⠒⠲⠶⢄⠄⠄⠄⠄⠄⠄
⠄⠄⠄⠄⠄⣠⣾⡿⠟⠋⠁⠄⢀⣀⡀⠤⣦⢰⣤⣶⢶⣤⣤⣈⣆⠄⠄⠄⠄⠄
⠄⠄⠄⠄⢰⠟⠁⠄⢀⣤⣶⣿⡿⠿⣿⣿⣊⡘⠲⣶⣷⣶⠶⠶⠶⠦⠤⡀⠄⠄
⠄⠔⠊⠁⠁⠄⠄⢾⡿⣟⡯⣖⠯⠽⠿⠛⠛⠭⠽⠊⣲⣬⠽⠟⠛⠛⠭⢵⣂⠄
⡎⠄⠄⠄⠄⠄⠄⠄⢙⡷⠋⣴⡆⠄⠐⠂⢸⣿⣿⡶⢱⣶⡇⠄⠐⠂⢹⣷⣶⠆
⡇⠄⠄⠄⠄⣀⣀⡀⠄⣿⡓⠮⣅⣀⣀⣐⣈⣭⠤⢖⣮⣭⣥⣀⣤⣤⣭⡵⠂⠄
⣤⡀⢠⣾⣿⣿⣿⣿⣷⢻⣿⣿⣶⣶⡶⢖⣢⣴⣿⣿⣟⣛⠿⠿⠟⣛⠉⠄⠄⠄
⣿⡗⣼⣿⣿⣿⣿⡿⢋⡘⠿⣿⣿⣷⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⡀⠄⠄
⣿⠱⢿⣿⣿⠿⢛⠰⣞⡛⠷⣬⣙⡛⠻⠿⠿⠿⣿⣿⣿⣿⣿⣿⣿⠿⠛⣓⡀⠄
⢡⣾⣷⢠⣶⣿⣿⣷⣌⡛⠷⣦⣍⣛⠻⠿⢿⣶⣶⣶⣦⣤⣴⣶⡶⠾⠿⠟⠁⠄
⣿⡟⣡⣿⣿⣿⣿⣿⣿⣿⣷⣦⣭⣙⡛⠓⠒⠶⠶⠶⠶⠶⠶⠶⠶⠿⠟⠄⠄⠄
⠿⡐⢬⣛⡻⠿⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⡶⠟⠃⠄⠄⠄⠄⠄⠄
⣾⣿⣷⣶⣭⣝⣒⣒⠶⠬⠭⠭⠭⠭⠭⠭⠭⣐⣒⣤⣄⡀⠄⠄⠄⠄⠄⠄⠄⠄
⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣦⠄⠄⠄⠄⠄⠄⠄
        "#,
    },
    AsciiArt {
        name: "Shrek",
        art: r#"
⢀⡴⠑⡄⠀⠀⠀⠀⠀⠀⠀⣀⣀⣤⣤⣤⣀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠸⡇⠀⠿⡀⠀⠀⠀⣀⡴⢿⣿⣿⣿⣿⣿⣿⣿⣷⣦⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠑⢄⣠⠾⠁⣀⣄⡈⠙⣿⣿⣿⣿⣿⣿⣿⣿⣆⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⢀⡀⠁⠀⠀⠈⠙⠛⠂⠈⣿⣿⣿⣿⣿⠿⡿⢿⣆⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⢀⡾⣁⣀⠀⠴⠂⠙⣗⡀⠀⢻⣿⣿⠭⢤⣴⣦⣤⣹⠀⠀⠀⢀⢴⣶⣆
⠀⠀⢀⣾⣿⣿⣿⣷⣮⣽⣾⣿⣥⣴⣿⣿⡿⢂⠔⢚⡿⢿⣿⣦⣴⣾⠁⠸⣼⡿
⠀⢀⡞⠁⠙⠻⠿⠟⠉⠀⠛⢹⣿⣿⣿⣿⣿⣌⢤⣼⣿⣾⣿⡟⠉⠀⠀⠀⠀⠀
⠀⣾⣷⣶⠇⠀⠀⣤⣄⣀⡀⠈⠻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡇⠀⠀⠀⠀⠀⠀
⠀⠉⠈⠉⠀⠀⢦⡈⢻⣿⣿⣿⣶⣶⣶⣶⣤⣽⡹⣿⣿⣿⣿⡇⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠉⠲⣽⡻⢿⣿⣿⣿⣿⣿⣿⣷⣜⣿⣿⣿⡇⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⣿⣷⣶⣮⣭⣽⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⣀⣀⣈⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠇⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠃⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠹⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠟⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠛⠻⠿⠿⠿⠿⠛⠉
        "#,
    },
    AsciiArt {
        name: "Termitype",
        art: r#"
 ⢀⠄⠄⠄⠄⠄⠄⠄⠄⢀⡀⠄⣰⣶⣦⡈⠄⠄⠄⠄⠄THINK
 ⠄⠄⠂⠄⠄⠄⠄⠄⠄⠄⣿⣖⣿⣷⣴⡄⠄⠄⠄⠄⠄YOU
 ⠄⠄⠄⠄⠁⠄⠄⠄⠄⣸⣿⣿⣿⠛⠩⠁⠄⠄⠄⠄⠄CAN
 ⠄⠄⠄⠄⠄⠄⣀⣤⣾⣿⣿⡏⠉⠄⠁⠄⠄⠄⠄⠄⠄BEAT
 ⠄⠄⢀⣴⣶⣿⣿⣿⣿⣿⡟⠺⡇⠄⢵⣤⣀⠄⠄⠄⠄ME
 ⠄⢠⣿⣿⣿⣿⣿⣿⣿⡏⠁⠄⣷⣀⠈⠙⠛⠑⠄⠄⠄IN
 ⠄⣼⣿⣿⣿⡇⠹⣿⣿⣿⡦⠄⠹⢿⡇⠄⠄⠄⠄⠄⠄TERMITYPE?
 ⠄⣿⣿⣿⣿⠁⢰⣤⣀⣀⣠⣔⢰⠄⠄⠄⠄⢀⡈⠄⠄
 ⢠⣿⣿⠟⠄⠄⢸⣿⣿⣿⣿⠏⢸⡆⠄⠐⠄⢸⣿⣌⠄
 ⢸⣿⣿⡆⠄⠄⢸⣿⡿⢿⡤⠄⠈⠄⠄⢀⠄⢰⣿⣿⡄
 ⠈⢿⣿⣷⠄⠄⠄⣿⣷⣦⠄⠐⠄⠄⠄⠘⠄⠘⢿⣿⡇
 ⠄⠈⠻⣿⣇⠠⠄⢀⡉⠄⠄⠄⠄⠄⢀⡆⠄⠄⠘⣿⡇
 ⠄⠄⠄⠘⣿⣧⢀⣿⣿⢷⣶⣶⣶⣾⢟⣾⣄⠄⡴⣿⡀
 ⠄⠄⠄⠄⠘⣿⣧⣝⣿⣷⣝⢿⣿⠇⢸⣿⣿⡎⡡⠋⠄
 ⠄⠄⠄⠄⠄⣝⠛⠋⠁⣿⣿⡎⢠⣴⣾⣿⣿⣷⠄⠄⠄
 ⠄⠄⠄⠄⢠⣿⣷⣾⣿⣿⣿⠁⠘⢿⣿⣿⣿⣿⡄⠄⠄
 ⠄⠄⠄⠄⢸⣿⣿⣿⣿⣿⠃⠄⠄⠈⣿⣿⣿⣿⡇⠄⠄
        "#,
    },
    AsciiArt {
        name: "Termitype 3D",
        art: r#"
 ______  ______   ______   __    __   __
/\__  _\/\  ___\ /\  == \ /\ "-./  \ /\ \
\/_/\ \/\ \  __\ \ \  __< \ \ \-./\ \\ \ \
   \ \_\ \ \_____\\ \_\ \_\\ \_\ \ \_\\ \_\
    \/_/  \/_____/ \/_/ /_/ \/_/  \/_/ \/_/
       ______  __  __   ______  ______
      /\__  _\/\ \_\ \ /\  == \/\  ___\
      \/_/\ \/\ \____ \\ \  _-/\ \  __\
         \ \_\ \/\_____\\ \_\   \ \_____\
          \/_/  \/_____/ \/_/    \/_____/

        "#,
    },
    AsciiArt {
        name: "Ubuntu",
        art: r#"
⣿⣿⣿⣿⣿⣿⣿⣿⠿⠛⠋⠉⠁⠀⠀⠀⠀⠈⠉⠙⠛⠿⣿⣿⣿⣿⣿⣿⣿⣿
⣿⣿⣿⣿⣿⠿⠋⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠙⠿⣿⣿⣿⣿⣿
⣿⣿⣿⡟⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⣾⣿⣦⠀⠀⠀⠈⢻⣿⣿⣿
⣿⣿⠏⠀⠀⠀⠀⠀⠀⠀⠀⢠⣶⣶⣾⣷⣶⣆⠸⣿⣿⡟⠀⠀⠀⠀⠀⠹⣿⣿
⣿⠃⠀⠀⠀⠀⠀⠀⣠⣾⣷⡈⠻⠿⠟⠻⠿⢿⣷⣤⣤⣄⠀⠀⠀⠀⠀⠀⠘⣿
⡏⠀⠀⠀⠀⠀⠀⣴⣿⣿⠟⠁⠀⠀⠀⠀⠀⠀⠈⠻⣿⣿⣦⠀⠀⠀⠀⠀⠀⢹
⠁⠀⠀⢀⣤⣤⡘⢿⣿⡏⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢹⣿⣿⡇⠀⠀⠀⠀⠀⠈
⠀⠀⠀⣿⣿⣿⡇⢸⣿⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢈⣉⣉⡁⠀⠀⠀⠀⠀⠀
⡀⠀⠀⠈⠛⠛⢡⣾⣿⣇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣸⣿⣿⡇⠀⠀⠀⠀⠀⢀
⣇⠀⠀⠀⠀⠀⠀⠻⣿⣿⣦⡀⠀⠀⠀⠀⠀⠀⢀⣴⣿⣿⠟⠀⠀⠀⠀⠀⠀⣸
⣿⡄⠀⠀⠀⠀⠀⠀⠙⢿⡿⢁⣴⣶⣦⣴⣶⣾⡿⠛⠛⠋⠀⠀⠀⠀⠀⠀⢠⣿
⣿⣿⣆⠀⠀⠀⠀⠀⠀⠀⠀⠘⠿⠿⢿⡿⠿⠏⢰⣿⣿⣧⠀⠀⠀⠀⠀⣰⣿⣿
⣿⣿⣿⣧⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠘⢿⣿⠟⠀⠀⠀⢀⣼⣿⣿⣿
⣿⣿⣿⣿⣿⣶⣄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣠⣶⣿⣿⣿⣿⣿
⣿⣿⣿⣿⣿⣿⣿⣿⣶⣤⣄⣀⡀⠀⠀⠀⠀⢀⣀⣠⣤⣶⣿⣿⣿⣿⣿⣿⣿⣿
        "#,
    },
    AsciiArt {
        name: "Windows7",
        art: r#"
      ,.=:!!t3Z3z.,
       :tt:::tt333EE3
       Et:::ztt33EEEL @Ee.,      ..,
      ;tt:::tt333EE7 ;EEEEEEttttt33#
     :Et:::zt333EEQ. $EEEEEttttt33QL
     it::::tt333EEF @EEEEEEttttt33F
    ;3=*^```"*4EEV :EEEEEEttttt33@.
    ,.=::::!t=., ` @EEEEEEtttz33QF
   ;::::::::zt33)   "4EEEtttji3P*
  :t::::::::tt33.:Z3z..  `` ,..g.
  i::::::::zt33F AEEEtttt::::ztF
 ;:::::::::t33V ;EEEttttt::::t3
 E::::::::zt33L @EEEtttt::::z3F
{3=*^```"*4E3) ;EEEtttt:::::tZ`
             ` :EEEEtttt::::z7
                 "VEzjt:;;z>*`
      "#,
    },
];
