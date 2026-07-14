use strum::Display;

#[derive(Debug, Default, Copy, Clone, Display)]
pub enum Lang { #[default]En, Ru, Uk }
impl Lang {
    #[inline]
    pub fn encode<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "en",
            Lang::Ru => "ru",
            Lang::Uk => "uk",
        }
    }
    #[inline]
    pub fn decode(lang_name: &str) -> Option<Self> {
        match lang_name.to_lowercase().as_str() {
            "en"|"eng"|"english" => Some(Self::En),
            "ru"|"rus"|"russian" => Some(Self::Ru),
            "uk"|"ukr"|"ukrainian" => Some(Self::Uk),
            _ => None,
        }
    }
}

pub mod errors {
    use super::Lang;

    pub const fn no_suitable_parameter<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "No suitable parameter",
            Lang::Ru => "Нет подходящего параметра",
            Lang::Uk => "Немає відповідного параметра",
        }
    }
    pub const fn no_links_to_filtering<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "No Links to filtering",
            Lang::Ru => "Нет ссылок для фильтрации",
            Lang::Uk => "Немає посилань на фільтрацію",
        }
    }
    pub const fn unable_to_download<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "Unable to download",
            Lang::Ru => "Невозможно скачать",
            Lang::Uk => "Неможливо завантажити",
        }
    }
    pub const fn no_link_to_extract_a_name<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "No Link to extract a name",
            Lang::Ru => "Нет ссылки для извлечения имени",
            Lang::Uk => "Немає посилання для отримання імені",
        }
    }
    pub const fn no_link_to_download<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "No Link to download",
            Lang::Ru => "Нет ссылки для скачивания",
            Lang::Uk => "Немає посилання для завантаження",
        }
    }
    pub const fn no_links_to_download<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "No Links to download",
            Lang::Ru => "Нет ссылок для скачивания",
            Lang::Uk => "Немає посилань для завантаження",
        }
    }
    pub const fn unable_to_connect<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "Unable to connect",
            Lang::Ru => "Невозможно подключиться",
            Lang::Uk => "Не може підключитися",
        }
    }
    pub const fn something_went_wrong<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "Something went wrong",
            Lang::Ru => "Что-то пошло не так",
            Lang::Uk => "Щось пішло не так",
        }
    }
    pub const fn no_date<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "No Date",
            Lang::Ru => "Нет даты",
            Lang::Uk => "Немає дати",
        }
    }
    pub const fn no_link_to_write<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "No Link to write to file",
            Lang::Ru => "Нет ссылки для записи в файл",
            Lang::Uk => "Немає посилання для запису у файл",
        }
    }
    pub const fn unable_to_download_a_file<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "Unable to download a file",
            Lang::Ru => "Невозможно скачать файл",
            Lang::Uk => "Неможливо завантажити файл",
        }
    }
    pub const fn wrong_path<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "Wrong Path",
            Lang::Ru => "Неправильный путь",
            Lang::Uk => "Неправильний шлях",
        }
    }
    pub const fn wrong_file_name<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "Wrong File Name",
            Lang::Ru => "Неправильное имя файла",
            Lang::Uk => "Неправильна назва файлу",
        }
    }
    pub const fn link_is_not_a_str_type<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "Link is not a 'str' type",
            Lang::Ru => "Ссылка не типа 'str'",
            Lang::Uk => "Посилання не має типу 'str'",
        }
    }
    pub const fn security_threat<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "Security Threat",
            Lang::Ru => "Угроза безопасности",
            Lang::Uk => "Загроза безпеці",
        }
    }
}
pub mod hints {
    use super::Lang;

    pub const fn genre<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "Genre",
            Lang::Ru => "Жанр",
            Lang::Uk => "Жанр",
        }
    }
    pub const fn quantity<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "Number of files (if PERIOD disabled)\nor days (if PERIOD enabled)",
            Lang::Ru => "Количество файлов (если Период выключен)\nили дней (если Период включен)",
            Lang::Uk => "Кількість файлів (якщо Крапка вимкнено)\nчи днів (якщо Крапка увімкнено)",
        }
    }
    pub const fn period<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "If enabled, search for all files within the specified number of recent days\nIf disabled, search for latest files",
            Lang::Ru => "Если включено, искать все файлы за указанное количество дней\nЕсли выключено, искать последние файлы",
            Lang::Uk => "Якщо увімкнено, шукати всі файли за вказану кількість днів\nЯкщо выключено, шукати останні файли",
        }
    }
    pub const fn lossless<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "If enabled, search for files with the following extensions: wav, flac, aiff\nIf disabled, only search for mp3",
            Lang::Ru => "Если включено, искать файлы со следующими расширениями: wav, flac, aiff\nЕсли выключено, искать только mp3",
            Lang::Uk => "Якщо увімкнено, шукати файли з такими розширеннями: wav, flac, aiff\nЯкщо выключено, шукати тільки mp3",
        }
    }
    pub const fn file_history<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "Enable/Disable history of downloaded files",
            Lang::Ru => "Включить/Выключить историю скачанных файлов",
            Lang::Uk => "Увімкнути/Вимкнути історію завантажених файлів",
        }
    }
    pub const fn overwrite_files<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "This checkbox only works if File History is DISABLED!\nIf enabled, overwrites existing files on download\nIf disabled, always create new files",
            Lang::Ru => "Этот флажок работает только если История Файлов ВЫКЛЮЧЕНА!\nЕсли включено, переписывать существующие файлы при скачивании\nЕсли выключено, всегда создавать новые файлы",
            Lang::Uk => "Цей прапорець працює, лише якщо Iсторія файлів ВИМКНЕНО!\nЯкщо увімкнено, переписувати існуючі файли під час скачування\nЯкщо выключено, завжди створювати нові файли",
        }
    }
    pub const fn switch_language<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "Switch language",
            Lang::Ru => "Переключить язык",
            Lang::Uk => "Змінити мову",
        }
    }
    pub const fn copy<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "Copy",
            Lang::Ru => "Копировать",
            Lang::Uk => "Копіювати",
        }
    }
    pub const fn donate<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "Donate",
            Lang::Ru => "Пожертвовать",
            Lang::Uk => "Пожертвувати",
        }
    }
    pub const fn pause<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "Pause",
            Lang::Ru => "Пауза",
            Lang::Uk => "Пауза",
        }
    }
}
pub mod ui_messages {
    use super::Lang;

    pub const fn matching_files_not_found<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "Matching files not found or already downloaded",
            Lang::Ru => "Подходящих файлов не найдено или уже скачаны",
            Lang::Uk => "Відповідні файли не знайдено або вже завантажено",
        }
    }
    pub const fn all_files_downloaded<'a>() -> &'a str { "100% - OK" }
    pub const fn searching<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "searching",
            Lang::Ru => "поиск",
            Lang::Uk => "пошук",
        }
    }
    pub const fn analysis<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "analysis",
            Lang::Ru => "анализ",
            Lang::Uk => "аналіз",
        }
    }
    pub const fn download_canceled<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "Download canceled",
            Lang::Ru => "Скачивание отменено",
            Lang::Uk => "Завантаження скасовано",
        }
    }
}
pub mod inscriptions {
    use super::Lang;

    pub const fn promodj_downloader_extended<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "PromoDJ Downloader - Last download was _ days ago",
            Lang::Ru => "PromoDJ Загрузчик - Последняя загрузка была _ дней назад",
            Lang::Uk => "PromoDJ Завантажувач - Останнє завантаження було _ днів тому",
        }
    }
    pub const fn period<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "Period",
            Lang::Ru => "Период",
            Lang::Uk => "Крапка",
        }
    }
    pub const fn lossless<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "Lossless",
            Lang::Ru => "Без потерь",
            Lang::Uk => "Без втрат",
        }
    }
    pub const fn file_history<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "File History",
            Lang::Ru => "История",
            Lang::Uk => "Iсторія",
        }
    }
    pub const fn overwrite_files<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "Overwrite files",
            Lang::Ru => "Переписывать",
            Lang::Uk => "Переписувати",
        }
    }
    pub const fn save_to<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "Save to",
            Lang::Ru => "Сохранить в",
            Lang::Uk => "Зберегти у",
        }
    }
    pub const fn download<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "Download",
            Lang::Ru => "Скачать",
            Lang::Uk => "Завантажити",
        }
    }
    pub const fn cancel<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "Cancel",
            Lang::Ru => "Отмена",
            Lang::Uk => "Відміна",
        }
    }
    pub const fn last_files<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "last files",
            Lang::Ru => "файлов",
            Lang::Uk => "файлів",
        }
    }
    pub const fn last_days<'a>(lang: Lang) -> &'a str {
        match lang {
            Lang::En => "last days",
            Lang::Ru => "дней",
            Lang::Uk => "днiв",
        }
    }
}