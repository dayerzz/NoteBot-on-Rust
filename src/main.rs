use teloxide::Bot;
use tokio::sync::Mutex;

use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{
        InlineKeyboardButton, InlineKeyboardMarkup,
        InlineKeyboardButtonKind, Me, ParseMode::MarkdownV2
    },
};

use std::{
    error::Error,
    sync::Arc, path::Path,
};

mod functions;
use functions::{
    create_folder, 
    create_file, 
    search_files_in_directory, 
    search_string_in_filenames,
    search_string_inside_files,
    escape_markdown_special_chars,
    create_message_and_keyboard,
    contains_invalid_chars
};

const MAX_TAG_TITLE_LENGTH: usize = 64;
const MAX_NOTE_TEXT_LENGTH: usize = 4000;
const TITLE_OFFSET: usize = SEARCH_TITLE_PREFIX.as_bytes().len() + 1;
const PHRASE_OFFSET: usize = SEARCH_PHRASE_PREFIX.as_bytes().len() + 1;

const NOTES_FOLDER: &str = "Заметки";
const SEARCH_PHRASE_PREFIX: &str = "Фраза:";
const SEARCH_TITLE_PREFIX: &str = "Заголовок:";
const TOKEN: &str = "token";


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    create_folder(NOTES_FOLDER);

    let bot = Bot::new(TOKEN);
    let changing = Arc::new(Mutex::new(String::new()));

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![changing])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

async fn message_handler(
    bot: Bot,
    msg: Message,
    _me: Me,
    changing: Arc<Mutex<String>>
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(text) = msg.text() {
        match text {
            "/start" => {
                bot.send_message(msg.chat.id, &format!(
                    "Приветствую вас, *{}*\\!👋\n\
                    Я специальный бот🤖 для хранения ваших заметок📝\\. С помощью меня вы легко сможете сохранить важную информацию и найти её\\.\n\
                    Добавить заметку вы можете с помощью следующего синтаксиса:\n\
                    `\\#тег` — тег является темой\\, группирующей все заметки\\,\n\
                    `Заголовок` — каждая заметка должна иметь свой заголовок\\,\n\
                    `Текст самой заметки` — информация\\, которую вы хотите сохранить\\.\n\
                    Давайте я покажу вам пример такой заметки:\n\
                    \\#игры\n\
                    Игра на вечер\n\
                    Хочу поиграть в Скайрим🎮\n\
                    После того как вы создали заметки\\, вы можете легко найти их🔎\\.\n\
                    Для поиска заметок используется следующий синтаксис:\n\
                    \\#тег — необязательный параметр\\, который поможет вам найти заметки именно с этим тегом\\.\n\
                    Фраза: фраза для поиска — так бот будет искать заметку по её содержимому\\, а не по заголовку\\.\n\
                    Заголовок: фраза для поиска — так бот будет искать заметки строго по их заголовкам\\.\n\
                    Это всё\\, что нужно вам знать\\, а теперь добавьте вашу первую заметку\\.",
                    msg.chat.first_name().unwrap())
                ).parse_mode(MarkdownV2).await.unwrap();
            },
            _ => {
                let mut note = changing.lock().await;

                if !note.is_empty() {
                    let _ = std::fs::write(&format!("{}\\{}", NOTES_FOLDER, &*note), text);
                    bot.send_message(msg.chat.id, "*Заметка изменена\\!*✍️").parse_mode(MarkdownV2).await.unwrap();
                    note.clear();
                }

                let lines: Vec<_> = text.split('\n').map(|s| s.trim()).collect();

                match lines.len() {
                    3 => {
                        if lines[0].chars().nth(0).unwrap() != '#' || lines[0].is_empty() || lines[1].is_empty() || lines[2].is_empty() {
                            bot.send_message(msg.chat.id,
                                "*Ошибка\\!*⚠️\nПозвольте напомнить вам синтаксис добавления заметки:\n\
                                `\\#тег` — тег является темой\\, группирующей все заметки\\,\n\
                                `Заголовок` — каждая заметка должна иметь свой заголовок\\,\n\
                                `Текст самой заметки` — информация\\, которую вы хотите сохранить\\."
                            ).parse_mode(MarkdownV2).await.unwrap();
                        }
                        else {
                            if lines[0].as_bytes().len() + lines[1].as_bytes().len() > MAX_TAG_TITLE_LENGTH {
                                bot.send_message(msg.chat.id,
                                    "*Ошибка\\!*⚠️\n\
                                    Тег или заголовок слишком длинные, чтобы бот мог сохранить заметку\\."
                                ).parse_mode(MarkdownV2).await.unwrap();
                            } else if lines[2].len() > MAX_NOTE_TEXT_LENGTH {
                                bot.send_message(msg.chat.id,
                                    "*Ошибка\\!*⚠️\n\
                                    Текст заметки слишкоком длинный, чтобы бот мог его сохранить\\."
                                ).parse_mode(MarkdownV2).await.unwrap();
                            } else {
                                let mut file_error = false;
                                if !contains_invalid_chars(lines[0]) {
                                    create_folder(&format!("{}\\{}", NOTES_FOLDER, lines[0]));
                                    if !contains_invalid_chars(lines[1]) {
                                        if !Path::new(&format!("{}\\{}\\{}.txt", NOTES_FOLDER, lines[0], lines[1])).exists() {
                                            create_file(&format!("{}\\{}\\{}.txt", NOTES_FOLDER, lines[0], lines[1]), lines[2]);
                                            bot.send_message(msg.chat.id, "*Заметка создана\\!*✅").parse_mode(MarkdownV2).await.unwrap();
                                        } else {
                                            bot.send_message(msg.chat.id,
                                                "*Ошибка\\!*⚠️\n\
                                                Заметка с таким заголовком уже существует\\."
                                            ).parse_mode(MarkdownV2).await.unwrap();
                                        }
                                    }
                                    else {
                                        file_error = true;
                                    }
                                    
                                }
                                else {
                                    file_error = true;
                                }
                                if file_error {
                                    bot.send_message(msg.chat.id,
                                        "*Ошибка\\!*⚠️\n\
                                        Не допускается использование таких символовов как:\n\
                                        \\, /, :, \\*, ?, \\\", \\<, \\>, \\|\\."
                                    ).parse_mode(MarkdownV2).await.unwrap();
                                }
                            }
                            
                        }
                    },
                    2 => {
                        if lines[0].chars().nth(0).unwrap() == '#' && !lines[1].is_empty() &&
                            (!lines[1].contains(SEARCH_PHRASE_PREFIX) || !lines[1].contains(&SEARCH_PHRASE_PREFIX.to_lowercase()) || 
                            !lines[1].contains(SEARCH_TITLE_PREFIX) || !lines[1].contains(&SEARCH_TITLE_PREFIX.to_lowercase())) {
                            if lines[0].chars().nth(0).unwrap() == '#' {
                                if lines[1].contains(SEARCH_TITLE_PREFIX) || lines[1].contains(&SEARCH_TITLE_PREFIX.to_lowercase()) {
                                    let search_str = &lines[1][TITLE_OFFSET..];
                                    let files = search_string_in_filenames(&search_str.trim().to_lowercase(), &format!("Заметки\\{}", lines[0]));
                                    if !files.is_empty() {
                                        let (message, inline_keyboard) = create_message_and_keyboard(files);
                                        bot.send_message(msg.chat.id, message).reply_markup(inline_keyboard).parse_mode(MarkdownV2).await.unwrap();
                                    } else {
                                        bot.send_message(msg.chat.id,
                                            "*Не найдено ни одной заметки\\.*"
                                        ).parse_mode(MarkdownV2).await.unwrap();
                                    }
                                }

                                if lines[1].contains(SEARCH_PHRASE_PREFIX) || lines[1].contains(&SEARCH_PHRASE_PREFIX.to_lowercase()) {
                                    let search_str = &lines[1][PHRASE_OFFSET..];
                                    let files = search_string_inside_files(&search_str.trim().to_lowercase(), &format!("Заметки\\{}", lines[0]));
                                    if !files.is_empty() {
                                        let (message, inline_keyboard) = create_message_and_keyboard(files);
                                        bot.send_message(msg.chat.id, message).reply_markup(inline_keyboard).parse_mode(MarkdownV2).await.unwrap();
                                    } else {
                                        bot.send_message(msg.chat.id,
                                            "*Не найдено ни одной заметки\\.*"
                                        ).parse_mode(MarkdownV2).await.unwrap();
                                    }
                                }
                            }
                        }
                        else {
                            bot.send_message(msg.chat.id,
                                "*Ошибка\\!*⚠️\nПозвольте напомнить вам синтаксис поиска заметок:\n\
                                \\#тег — необязательный параметр\\, который поможет вам найти заметки именно с этим тегом\\.\n\
                                Фраза: фраза для поиска — так бот будет искать заметку по её содержимому\\, а не по заголовку\\.\n\
                                Заголовок: фраза для поиска — так бот будет искать заметки строго по их заголовкам\\."
                            ).parse_mode(MarkdownV2).await.unwrap();
                        }
                    },
                    1 => {
                        if lines[0].chars().nth(0).unwrap() == '#' {
                            let files = search_files_in_directory("", &format!("{}\\{}", NOTES_FOLDER, lines[0]));
                            if !files.is_empty() {
                                let (message, inline_keyboard) = create_message_and_keyboard(files);
                                bot.send_message(msg.chat.id, message).reply_markup(inline_keyboard).parse_mode(MarkdownV2).await.unwrap();
                            } else {
                                bot.send_message(msg.chat.id,
                                    "*Не найдено ни одной заметки\\.*"
                                ).parse_mode(MarkdownV2).await.unwrap();
                            }
                        }
                        if lines[0].contains(SEARCH_TITLE_PREFIX) {
                            let search_str = &lines[0][TITLE_OFFSET..];
                            let files = search_string_in_filenames(&search_str.trim().to_lowercase(), NOTES_FOLDER);
                            if !files.is_empty() {
                                let (message, inline_keyboard) = create_message_and_keyboard(files);
                                bot.send_message(msg.chat.id, message).reply_markup(inline_keyboard).parse_mode(MarkdownV2).await.unwrap();
                            } else {
                                bot.send_message(msg.chat.id,
                                    "*Не найдено ни одной заметки\\.*"
                                ).parse_mode(MarkdownV2).await.unwrap();
                            }
                        }
                        if lines[0].contains(SEARCH_PHRASE_PREFIX) {
                            let search_str = &lines[0][PHRASE_OFFSET..];
                            let files = search_string_inside_files(&search_str.trim().to_lowercase(), NOTES_FOLDER);
                            if !files.is_empty() {
                                let (message, inline_keyboard) = create_message_and_keyboard(files);
                                bot.send_message(msg.chat.id, message).reply_markup(inline_keyboard).parse_mode(MarkdownV2).await.unwrap();
                            } else {
                                bot.send_message(msg.chat.id,
                                    "*Не найдено ни одной заметки\\.*"
                                ).parse_mode(MarkdownV2).await.unwrap();
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

async fn callback_handler(bot: Bot, q: CallbackQuery, changing: Arc<Mutex<String>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(text) = q.data {
        if text.contains("#") && text.contains("\\") && text.chars().nth(0).unwrap() != 'd' && text.chars().nth(0).unwrap() != 'w' {
            let data_of_notes: Vec<_> = text.split("\\").collect();

            let text_from_file = &std::fs::read_to_string(format!("{}\\{}", NOTES_FOLDER, text.clone())).unwrap_or_default();

            if !text_from_file.is_empty() {
                let message = String::from(format!("*Ваша заметка📝:*\n*Тег:* {}\n*Заголовок:* {}\n*Текст:*\n`{}`",
                    escape_markdown_special_chars(&data_of_notes[0]),
                    escape_markdown_special_chars(&data_of_notes[1].replace(".txt", "")),
                    escape_markdown_special_chars(text_from_file)
                ));

                let mut inline_keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];
                let mut chunk = vec![];
                chunk.push(InlineKeyboardButton::new("Удалить", InlineKeyboardButtonKind::CallbackData(format!("d{}", text.clone()))));
                chunk.push(InlineKeyboardButton::new("Изменить", InlineKeyboardButtonKind::CallbackData(format!("w{}", text.clone()))));
                inline_keyboard.push(chunk);

                bot.send_message(q.message.clone().unwrap().chat.id, message)
                    .reply_markup(InlineKeyboardMarkup::new(inline_keyboard))
                    .parse_mode(MarkdownV2)
                    .await
                    .unwrap();
            }
        }
        if text.chars().nth(0).unwrap() == 'd' {
            let _ = std::fs::remove_file(format!("{}\\{}", NOTES_FOLDER, &text[1..]));
            bot.edit_message_text(q.message.clone().unwrap().chat.id, q.message.clone().unwrap().id, "*Заметка удалена\\!*♻️")
                .parse_mode(MarkdownV2)
                .await
                .unwrap();
        }
        if text.chars().nth(0).unwrap() == 'w' {
            bot.edit_message_text(q.message.clone().unwrap().chat.id, q.message.unwrap().id, "*Введите новый текст заметки:*")
                .parse_mode(MarkdownV2)
                .await
                .unwrap();
            *changing.lock().await = text[1..].to_string();
        }

        bot.answer_callback_query(q.id).await?;
    }

    Ok(())
}
