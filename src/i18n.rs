use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct I18n {
    pub translations: HashMap<String, String>,
}

impl I18n {
    pub fn new() -> Self {
        let mut translations = HashMap::new();
        
        translations.insert("app_title".to_string(), "База знаний".to_string());
        translations.insert("search_placeholder".to_string(), "Поиск по заголовкам, содержанию, тегам...".to_string());
        translations.insert("new_article".to_string(), "Новая статья".to_string());
        translations.insert("edit".to_string(), "Редактировать".to_string());
        translations.insert("delete".to_string(), "Удалить".to_string());
        translations.insert("save".to_string(), "Сохранить".to_string());
        translations.insert("cancel".to_string(), "Отмена".to_string());
        translations.insert("title".to_string(), "Заголовок".to_string());
        translations.insert("content".to_string(), "Содержание".to_string());
        translations.insert("tags".to_string(), "Теги (через запятую)".to_string());
        translations.insert("author".to_string(), "Автор".to_string());
        translations.insert("pinned".to_string(), "Закреплено".to_string());
        translations.insert("created".to_string(), "Создано".to_string());
        translations.insert("updated".to_string(), "Обновлено".to_string());
        translations.insert("no_pages".to_string(), "Статей пока нет. Создайте первую!".to_string());
        translations.insert("loading".to_string(), "Загрузка...".to_string());
        translations.insert("theme".to_string(), "Тема".to_string());
        translations.insert("all_articles".to_string(), "Все статьи".to_string());
        translations.insert("by".to_string(), "от".to_string());
        translations.insert("search_results".to_string(), "Найдено".to_string());
        translations.insert("of".to_string(), "из".to_string());
        translations.insert("not_found".to_string(), "Страница не найдена".to_string());
        translations.insert("back_to_list".to_string(), "Вернуться к списку".to_string());
        translations.insert("pin".to_string(), "Закрепить".to_string());
        translations.insert("unpin".to_string(), "Открепить".to_string());
        
        I18n { translations }
    }
    
    pub fn get(&self, key: &str) -> String {
        self.translations.get(key).unwrap_or(&key.to_string()).clone()
    }
}