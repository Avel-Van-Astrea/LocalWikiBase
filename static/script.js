let currentTheme = localStorage.getItem('wiki-theme') || 'light';
let allPages = [];
let translations = {};
let editingPageId = null;
let sidebarOpen = true;

async function loadTranslations() {
    const res = await fetch('/api/i18n');
    translations = await res.json();
    updateUI();
}

function updateUI() {
    document.querySelectorAll('[data-i18n]').forEach(el => {
        const key = el.getAttribute('data-i18n');
        if (translations[key]) {
            el.textContent = translations[key];
        }
    });
    
    document.querySelectorAll('[data-i18n-placeholder]').forEach(el => {
        const key = el.getAttribute('data-i18n-placeholder');
        if (translations[key]) {
            el.placeholder = translations[key];
        }
    });
    
    document.title = translations.app_title || 'Корпоративная база знаний';
}

function toggleSidebar() {
    const sidebar = document.getElementById('sidebar');
    const mainContent = document.getElementById('mainContent');
    const openBtn = document.getElementById('sidebarOpenBtn');
    
    sidebarOpen = !sidebarOpen;
    
    if (sidebarOpen) {
        sidebar.classList.remove('collapsed');
        mainContent.classList.remove('expanded');
        openBtn.classList.add('hidden');
    } else {
        sidebar.classList.add('collapsed');
        mainContent.classList.add('expanded');
        openBtn.classList.remove('hidden');
    }
}

function toggleSettings() {
    const dropdown = document.getElementById('settingsDropdown');
    dropdown.classList.toggle('open');
}

// Закрываем меню настроек при клике вне его
document.addEventListener('click', function(e) {
    const menu = document.querySelector('.settings-menu');
    if (menu && !menu.contains(e.target)) {
        document.getElementById('settingsDropdown').classList.remove('open');
    }
});

async function loadPages() {
    try {
        const res = await fetch('/api/pages');
        if (!res.ok) throw new Error('Ошибка загрузки');
        allPages = await res.json();
        renderPages(allPages);
        updateResultsInfo(allPages.length, allPages.length);
    } catch (error) {
        console.error('Ошибка загрузки страниц:', error);
        document.getElementById('pagesGrid').innerHTML = '<p>Ошибка загрузки страниц</p>';
    }
}

function updateResultsInfo(count, total) {
    const info = document.getElementById('resultsInfo');
    if (count === total) {
        info.textContent = `${translations.all_articles || 'Все статьи'}: ${count}`;
    } else {
        info.textContent = `${translations.search_results || 'Найдено'}: ${count} ${translations.of || 'из'} ${total}`;
    }
}

function renderPages(pages) {
    const grid = document.getElementById('pagesGrid');
    
    if (!pages || pages.length === 0) {
        grid.innerHTML = `<div style="grid-column: 1/-1; text-align: center; padding: 60px 20px; color: var(--text-secondary);">
            <p style="font-size: 18px;">${translations.no_pages || 'Статей пока нет. Создайте первую!'}</p>
        </div>`;
        return;
    }
    
    grid.innerHTML = pages.map(page => {
        const langLabel = page.lang === 'ru' ? 'RU' : 'EN';
        const langClass = page.lang === 'ru' ? 'lang-ru' : 'lang-en';
        
        return `
        <div class="page-card ${page.pinned ? 'pinned' : ''}" onclick="viewPage(${page.id})">
            <div class="page-title">
                ${page.pinned ? `<span class="pinned-badge">${translations.pinned || 'Закреплено'}</span> ` : ''}
                ${page.title}
                <span class="lang-badge ${langClass}">${langLabel}</span>
            </div>
            <div class="page-tags">
                ${page.tags.map(tag => `<span class="tag">#${tag}</span>`).join('')}
            </div>
            <div class="page-meta">
                ${translations.by || 'от'} ${page.author} • ${new Date(page.updated_at).toLocaleDateString('ru-RU')}
            </div>
        </div>
    `}).join('');
}

// Отображение результатов поиска в сайдбаре
function renderSearchResults(results) {
    const list = document.getElementById('searchResultsList');
    
    if (!results || results.length === 0) {
        list.innerHTML = `<div style="padding: 12px; color: var(--text-secondary); text-align: center; font-size: 14px;">
            ${translations.no_pages || 'Ничего не найдено'}
        </div>`;
        return;
    }
    
    list.innerHTML = results.map(page => {
        const langLabel = page.lang === 'ru' ? 'RU' : 'EN';
        // Создаём сниппет
        const snippet = page.content.length > 100 ? page.content.substring(0, 100) + '...' : page.content;
        
        return `
        <div class="search-result-item" onclick="viewPage(${page.id})">
            <div class="result-title">${page.title}</div>
            <div class="result-meta">
                <span class="lang-badge-small">${langLabel}</span>
                ${page.tags.map(tag => `#${tag}`).join(' ')}
            </div>
            <div class="result-snippet">${snippet}</div>
        </div>
    `}).join('');
}

async function viewPage(id) {
    try {
        const res = await fetch(`/api/page/${id}`);
        if (!res.ok) throw new Error('Страница не найдена');
        const page = await res.json();
        
        const langLabel = page.lang === 'ru' ? 'RU' : 'EN';
        const langClass = page.lang === 'ru' ? 'lang-ru' : 'lang-en';
        
        document.getElementById('viewContent').innerHTML = `
            <div style="display: flex; justify-content: space-between; align-items: start; margin-bottom: 10px;">
                <h1 style="margin: 0;">${page.title}</h1>
                <div style="display: flex; gap: 8px; align-items: center;">
                    <span class="lang-badge ${langClass}" style="font-size: 12px; padding: 4px 12px; border-radius: 12px;">${langLabel}</span>
                    <button onclick="openEditPage(${page.id})" 
                            style="padding: 4px 12px; background: var(--accent-color); color: white; 
                                   border: none; border-radius: 6px; cursor: pointer; font-size: 12px;">
                        ✏️ ${translations.edit || 'Редактировать'}
                    </button>
                    <button onclick="togglePinned(${page.id}, ${!page.pinned})" 
                            style="padding: 4px 12px; background: ${page.pinned ? 'var(--pinned-color)' : 'var(--bg-hover)'}; 
                                   color: ${page.pinned ? 'white' : 'var(--text-primary)'}; 
                                   border: 1px solid var(--border-color); border-radius: 6px; cursor: pointer; font-size: 12px;">
                        ${page.pinned ? '📌 ' + (translations.unpin || 'Открепить') : '📍 ' + (translations.pin || 'Закрепить')}
                    </button>
                </div>
            </div>
            <div style="color: var(--text-secondary); margin: 10px 0;">
                ${translations.by || 'от'} ${page.author} • ${translations.created || 'Создано'}: ${new Date(page.created_at).toLocaleDateString('ru-RU')}
                • ${translations.updated || 'Обновлено'}: ${new Date(page.updated_at).toLocaleDateString('ru-RU')}
                ${page.pinned ? ` • ${translations.pinned || 'Закреплено'}` : ''}
            </div>
            <div class="page-tags">
                ${page.tags.map(tag => `<span class="tag">#${tag}</span>`).join('')}
            </div>
            <hr style="margin: 20px 0; border: none; border-top: 2px solid var(--border-color);">
            <div class="markdown-body">${page.html}</div>
        `;
        
        document.getElementById('viewModal').classList.add('active');
    } catch (error) {
        alert('Ошибка загрузки статьи: ' + error.message);
    }
}

async function openEditPage(id) {
    try {
        const res = await fetch(`/api/page/${id}`);
        if (!res.ok) throw new Error('Страница не найдена');
        const page = await res.json();
        
        editingPageId = id;
        
        document.getElementById('editPageTitle').value = page.title;
        document.getElementById('editPageContent').value = page.content;
        document.getElementById('editPageTags').value = page.tags.join(', ');
        document.getElementById('editPageAuthor').value = page.author;
        
        document.getElementById('editModal').classList.add('active');
        document.getElementById('viewModal').classList.remove('active');
    } catch (error) {
        alert('Ошибка загрузки статьи: ' + error.message);
    }
}

async function updatePage(e) {
    e.preventDefault();
    
    const data = {
        title: document.getElementById('editPageTitle').value,
        content: document.getElementById('editPageContent').value,
        tags: document.getElementById('editPageTags').value.split(',').map(t => t.trim()).filter(Boolean),
        author: document.getElementById('editPageAuthor').value,
        lang: ''
    };
    
    try {
        const res = await fetch(`/api/page/${editingPageId}`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(data),
        });
        
        if (res.ok) {
            closeEditModal();
            await loadPages();
            const modal = document.getElementById('viewModal');
            if (modal.classList.contains('active')) {
                await viewPage(editingPageId);
            }
            editingPageId = null;
        } else {
            const error = await res.text();
            alert('Ошибка при обновлении статьи: ' + error);
        }
    } catch (error) {
        alert('Ошибка: ' + error.message);
    }
}

async function togglePinned(id, pinned) {
    try {
        const res = await fetch(`/api/page/${id}/pinned`, {
            method: 'PATCH',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ pinned })
        });
        
        if (res.ok) {
            await loadPages();
            const modal = document.getElementById('viewModal');
            if (modal.classList.contains('active')) {
                await viewPage(id);
            }
        } else {
            alert('Ошибка при изменении статуса');
        }
    } catch (error) {
        alert('Ошибка: ' + error.message);
    }
}

async function createPage(e) {
    e.preventDefault();
    
    const data = {
        title: document.getElementById('pageTitle').value,
        content: document.getElementById('pageContent').value,
        tags: document.getElementById('pageTags').value.split(',').map(t => t.trim()).filter(Boolean),
        author: document.getElementById('pageAuthor').value,
        lang: ''
    };
    
    try {
        const res = await fetch('/api/page', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(data),
        });
        
        if (res.ok) {
            closeNewPageModal();
            loadPages();
            document.getElementById('newPageForm').reset();
        } else {
            const error = await res.text();
            alert('Ошибка при создании страницы: ' + error);
        }
    } catch (error) {
        alert('Ошибка: ' + error.message);
    }
}

// Поиск в сайдбаре с отображением результатов
document.getElementById('searchInput')?.addEventListener('input', function() {
    const q = this.value.trim().toLowerCase();
    const searchResultsList = document.getElementById('searchResultsList');
    const resultsInfo = document.getElementById('resultsInfo');
    
    if (q.length < 1) {
        // Показываем все статьи
        renderSearchResults(allPages);
        updateResultsInfo(allPages.length, allPages.length);
        return;
    }
    
    const results = allPages.filter(page => {
        const titleMatch = page.title.toLowerCase().includes(q);
        const tagsMatch = page.tags.some(tag => tag.toLowerCase().includes(q));
        const contentMatch = page.content.toLowerCase().includes(q);
        const authorMatch = page.author.toLowerCase().includes(q);
        return titleMatch || tagsMatch || contentMatch || authorMatch;
    });
    
    renderSearchResults(results);
    resultsInfo.textContent = `${translations.search_results || 'Найдено'}: ${results.length} ${translations.of || 'из'} ${allPages.length}`;
});

function setTheme(theme) {
    currentTheme = theme;
    localStorage.setItem('wiki-theme', theme);
    document.documentElement.setAttribute('data-theme', theme);
    
    document.querySelectorAll('.theme-toggle button').forEach(btn => {
        btn.classList.toggle('active', btn.dataset.theme === theme);
    });
}

function openNewPageModal() {
    document.getElementById('newPageModal').classList.add('active');
}

function closeNewPageModal() {
    document.getElementById('newPageModal').classList.remove('active');
}

function closeEditModal() {
    document.getElementById('editModal').classList.remove('active');
    editingPageId = null;
}

function closeViewModal() {
    document.getElementById('viewModal').classList.remove('active');
}

document.getElementById('viewModal')?.addEventListener('click', function(e) {
    if (e.target === this) closeViewModal();
});

document.getElementById('newPageModal')?.addEventListener('click', function(e) {
    if (e.target === this) closeNewPageModal();
});

document.getElementById('editModal')?.addEventListener('click', function(e) {
    if (e.target === this) closeEditModal();
});

async function init() {
    setTheme(currentTheme);
    await loadTranslations();
    await loadPages();
    // Показываем все статьи в результатах поиска по умолчанию
    renderSearchResults(allPages);
}

document.addEventListener('DOMContentLoaded', init);