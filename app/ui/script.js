/* ===================================================
   COMIC ORDER - TAURI INTERACTION SCRIPT
   =================================================== */

// ESTADO GLOBAL DE LA APLICACIÓN
let collections = [];
let currentCollectionIndex = 0;
let activeCollectionComics = [];
let activeCollectionId = null;
let currentReadingComicId = null;
let currentReadingComicIndex = -1;
let currentReadingPage = 0;
let currentReadingTotalPages = 1;

// ESTADO DE ZOOM Y PANEO DEL VISOR
let readerZoom = 1.0;
let readerPanX = 0;
let readerPanY = 0;
let isPanning = false;
let startPanX = 0;
let startPanY = 0;

// HELPER DE COMUNICACIÓN IPC CON TAURI
async function invokeBackend(command, args = {}) {
  if (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke) {
    try {
      return await window.__TAURI__.core.invoke(command, args);
    } catch (err) {
      console.error(`Error ejecutando comando Tauri '${command}':`, err);
      throw err;
    }
  } else {
    console.warn(`Tauri no detectado. Modo mock para desarrollo web (${command})`);
    return mockBackend(command, args);
  }
}

// HELPER PARA CONVERTIR RUTAS LOCALES A URLS DE ASSET COMPATIBLES CON TAURI
function toAssetUrl(filePath) {
  if (!filePath) return '';
  const clean = filePath.replace(/\\/g, '/');
  if (clean.startsWith('data:') || clean.startsWith('blob:') || clean.startsWith('http://') || clean.startsWith('https://') || clean.startsWith('assets/')) {
    return clean;
  }
  if (window.__TAURI__ && window.__TAURI__.core && typeof window.__TAURI__.core.convertFileSrc === 'function') {
    try {
      return window.__TAURI__.core.convertFileSrc(clean);
    } catch (e) {
      console.warn('convertFileSrc:', e);
    }
  }
  return `http://asset.localhost/${clean}`;
}

function playSound(_type) {
  // Audio deshabilitado para respetar la estética sobria
}

// GESTIÓN DE MODALES CON TRANSICIONES DE DESVANECIDO
function openModal(modalId) {
  const modal = document.getElementById(modalId);
  if (modal) {
    modal.classList.add('active');
  }
}

function closeModal(modalId) {
  const modal = document.getElementById(modalId);
  if (modal) {
    modal.classList.remove('active');
  }
}

function closeAllModals() {
  document.querySelectorAll('.dossier-modal-overlay').forEach(modal => {
    modal.classList.remove('active');
  });
}

// INICIALIZACIÓN AL CARGAR LA PÁGINA
document.addEventListener('DOMContentLoaded', async () => {
  setupEventListeners();
  setupReaderZoomAndPan();
  setupGlitchCanvas();
  await loadCollections();
  await loadServerStatus();
});

// CONFIGURACIÓN DE ESCUCHADORES DE EVENTOS
function setupEventListeners() {
  // Botones de Navegación a Inicio
  document.getElementById('nav-home-btn').addEventListener('click', (e) => {
    e.preventDefault();
    closeAllModals();
    showHomeView();
  });
  document.getElementById('nav-home-logo').addEventListener('click', (e) => {
    e.preventDefault();
    closeAllModals();
    showHomeView();
  });
  document.getElementById('btn-back-to-home').addEventListener('click', (e) => {
    e.preventDefault();
    showHomeView();
  });

  // Botón LEER MÁS (Abre la Vista de Cómics de la Colección Activa)
  document.getElementById('read-more-btn').addEventListener('click', () => {
    playSound('click');
    if (collections.length > 0) {
      const col = collections[currentCollectionIndex];
      showCollectionComicsView(col.id);
    }
  });

  // Botón Nueva Colección (Navbar y Estado Vacío)
  document.getElementById('btn-open-new-collection').addEventListener('click', (e) => {
    e.preventDefault();
    openCollectionFormModal(null);
  });
  document.getElementById('btn-empty-create-collection').addEventListener('click', () => {
    openCollectionFormModal(null);
  });

  // Botón Re-escanear Biblioteca Global
  document.getElementById('btn-scan-library').addEventListener('click', async (e) => {
    e.preventDefault();
    playSound('thwip');
    try {
      const result = await invokeBackend('scan_monitored_paths');
      alert(`Escaneo completado: ${result} cómics actualizados`);
      await loadCollections();
      
      // Solo refrescar la grilla de cómics si el usuario ya se encuentra en la vista de colección
      const comicsView = document.getElementById('collection-comics-view');
      if (comicsView && comicsView.style.display !== 'none' && activeCollectionId) {
        await refreshActiveCollectionComics();
      }
    } catch (err) {
      alert(`Error al escanear: ${err}`);
    }
  });

  // Botón Re-escanear Carpeta desde Estado Vacío de Colección
  document.getElementById('btn-rescan-collection').addEventListener('click', async () => {
    const btn = document.getElementById('btn-rescan-collection');
    btn.disabled = true;
    btn.textContent = '🔄 Escaneando...';
    try {
      await invokeBackend('scan_monitored_paths');
      await refreshActiveCollectionComics();
    } catch (err) {
      alert(`Error al escanear: ${err}`);
    } finally {
      btn.disabled = false;
      btn.textContent = '🔄 Re-escanear Carpeta';
    }
  });

  // Botón Modal Servidor QR
  document.getElementById('btn-open-qr-modal').addEventListener('click', async (e) => {
    e.preventDefault();
    await openQrModal();
  });

  // Botón Modal Dispositivos de Confianza
  document.getElementById('btn-open-devices-modal').addEventListener('click', async (e) => {
    e.preventDefault();
    await openDevicesModal();
  });

  // Cerrar modales al hacer clic en botones de cierre o backdrop
  document.querySelectorAll('.modal-close-trigger').forEach(btn => {
    btn.addEventListener('click', () => closeAllModals());
  });
  document.querySelectorAll('.dossier-modal-overlay').forEach(overlay => {
    overlay.addEventListener('click', (e) => {
      if (e.target === overlay) closeAllModals();
    });
  });

  // Formulario de Colección
  document.getElementById('collection-edit-form').addEventListener('submit', async (e) => {
    e.preventDefault();
    await handleSaveCollection();
  });

  // Selectores de Archivo Nativo y Carpetas
  document.getElementById('btn-pick-bg').addEventListener('click', async () => {
    const path = await invokeBackend('pick_image_file');
    if (path) document.getElementById('form-bg-path').value = path;
  });
  document.getElementById('btn-pick-hero').addEventListener('click', async () => {
    const path = await invokeBackend('pick_image_file');
    if (path) document.getElementById('form-hero-path').value = path;
  });
  document.getElementById('btn-pick-comics-folder').addEventListener('click', async () => {
    const path = await invokeBackend('pick_folder');
    if (path) document.getElementById('form-comics-path').value = path;
  });

  // Botón Editar Colección Actual desde el Dossier
  document.getElementById('btn-edit-current-collection').addEventListener('click', () => {
    if (collections[currentCollectionIndex]) {
      closeAllModals();
      openCollectionFormModal(collections[currentCollectionIndex]);
    }
  });

  // Añadir Dispositivo de Confianza (Clic o tecla Enter)
  const handleAddDevice = async () => {
    const nameInput = document.getElementById('input-new-device-name');
    const name = nameInput.value.trim();
    if (!name) return;
    try {
      await invokeBackend('add_trusted_device', { deviceName: name });
      nameInput.value = '';
      await openDevicesModal();
    } catch (err) {
      alert(`Error al añadir dispositivo: ${err}`);
    }
  };

  document.getElementById('btn-add-device').addEventListener('click', handleAddDevice);
  const deviceInput = document.getElementById('input-new-device-name');
  if (deviceInput) {
    deviceInput.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') {
        e.preventDefault();
        handleAddDevice();
      }
    });
  }

  // Visor de Cómic - Controles de Navegación
  document.getElementById('reader-close-btn').addEventListener('click', () => {
    closeReader();
  });
  document.getElementById('btn-reader-back-grid').addEventListener('click', () => {
    closeReader();
  });
  document.getElementById('btn-reader-next-comic').addEventListener('click', () => {
    openNextComicInReader();
  });

  document.getElementById('reader-prev-btn').addEventListener('click', () => changeReaderPage(-1));
  document.getElementById('reader-next-btn').addEventListener('click', () => changeReaderPage(1));

  // Botones de Zoom en Barra Superior
  const btnZoomIn = document.getElementById('reader-zoom-in-btn');
  if (btnZoomIn) {
    btnZoomIn.addEventListener('click', () => setReaderZoom(readerZoom * 1.25));
  }
  const btnZoomOut = document.getElementById('reader-zoom-out-btn');
  if (btnZoomOut) {
    btnZoomOut.addEventListener('click', () => setReaderZoom(readerZoom * 0.8));
  }
  const zoomLevelTag = document.getElementById('reader-zoom-level');
  if (zoomLevelTag) {
    zoomLevelTag.addEventListener('click', () => resetReaderZoom());
  }

  // Navegación con teclado para el lector
  window.addEventListener('keydown', (e) => {
    const reader = document.getElementById('comic-reader');
    if (reader && reader.classList.contains('active')) {
      if (e.key === 'ArrowRight' || e.key === 'PageDown' || e.key === ' ') {
        changeReaderPage(1);
      } else if (e.key === 'ArrowLeft' || e.key === 'PageUp') {
        changeReaderPage(-1);
      } else if (e.key === '+' || e.key === '=') {
        setReaderZoom(readerZoom * 1.2);
      } else if (e.key === '-' || e.key === '_') {
        setReaderZoom(readerZoom * 0.8);
      } else if (e.key === '0') {
        resetReaderZoom();
      } else if (e.key === 'Escape') {
        closeReader();
      }
    }
  });
}

// CONFIGURACIÓN DE ZOOM Y PANEO INTERACTIVO EN EL VISOR
function setupReaderZoomAndPan() {
  const viewport = document.getElementById('reader-viewport');
  const img = document.getElementById('reader-page-img');
  if (!viewport) return;

  // Deshabilitar comportamiento de arrastre fantasma nativo del navegador
  if (img) {
    img.setAttribute('draggable', 'false');
    img.addEventListener('dragstart', (e) => e.preventDefault());
  }

  // Zoom continuo con la rueda del ratón (Scroll del mouse)
  viewport.addEventListener('wheel', (e) => {
    e.preventDefault();
    const zoomFactor = e.deltaY < 0 ? 1.15 : 0.87;
    setReaderZoom(readerZoom * zoomFactor);
  }, { passive: false });

  // Paneo interactivo: EXCLUSIVAMENTE al mantener presionado el clic con zoom activo
  viewport.addEventListener('mousedown', (e) => {
    // Solo botón principal (izquierdo) y cuando exista zoom activo
    if (e.button !== 0 || readerZoom <= 1.05) return;
    
    e.preventDefault();
    isPanning = true;
    startPanX = e.clientX - readerPanX;
    startPanY = e.clientY - readerPanY;
    viewport.classList.add('is-dragging');
  });

  window.addEventListener('mousemove', (e) => {
    if (!isPanning) return;

    // Si el botón izquierdo no está actualmente presionado, detener de inmediato el arrastre
    if ((e.buttons & 1) !== 1) {
      isPanning = false;
      viewport.classList.remove('is-dragging');
      return;
    }

    e.preventDefault();
    readerPanX = e.clientX - startPanX;
    readerPanY = e.clientY - startPanY;
    applyReaderTransform(false);
  });

  const stopPanning = () => {
    if (isPanning) {
      isPanning = false;
      viewport.classList.remove('is-dragging');
    }
  };

  window.addEventListener('mouseup', stopPanning);
  window.addEventListener('blur', stopPanning);
  viewport.addEventListener('mouseleave', (e) => {
    // Si sale del viewport y no tiene presionado el clic
    if ((e.buttons & 1) !== 1) {
      stopPanning();
    }
  });

  // Doble clic para alternar entre tamaño ajustado (100%) y aumento (200%)
  viewport.addEventListener('dblclick', (e) => {
    e.preventDefault();
    if (readerZoom > 1.05) {
      resetReaderZoom();
    } else {
      setReaderZoom(2.0);
    }
  });
}

function updateReaderCursorState() {
  const viewport = document.getElementById('reader-viewport');
  if (!viewport) return;
  if (readerZoom > 1.05) {
    viewport.classList.add('can-pan');
  } else {
    viewport.classList.remove('can-pan');
    viewport.classList.remove('is-dragging');
    isPanning = false;
  }
}

function applyReaderTransform(animate = false) {
  const img = document.getElementById('reader-page-img');
  const zoomTag = document.getElementById('reader-zoom-level');
  if (!img) return;

  if (animate) {
    img.style.transition = 'transform 0.12s ease-out, opacity 0.2s ease-out';
  } else {
    img.style.transition = 'none';
  }

  img.style.transform = `translate(${readerPanX}px, ${readerPanY}px) scale(${readerZoom})`;
  if (zoomTag) {
    zoomTag.textContent = `${Math.round(readerZoom * 100)}%`;
  }
  updateReaderCursorState();
}

function resetReaderZoom() {
  readerZoom = 1.0;
  readerPanX = 0;
  readerPanY = 0;
  isPanning = false;
  applyReaderTransform(true);
}

function setReaderZoom(newZoom) {
  const clampedZoom = Math.min(Math.max(newZoom, 0.5), 5.0);
  if (clampedZoom <= 1.05) {
    readerPanX = 0;
    readerPanY = 0;
    isPanning = false;
  }
  readerZoom = clampedZoom;
  applyReaderTransform(true);
}

// CAMBIO DE VISTAS (HOME VS CATÁLOGO DE CÓMIC)
function showHomeView() {
  activeCollectionId = null;
  document.getElementById('collection-comics-view').style.display = 'none';
  const dock = document.querySelector('.bottom-dock');

  if (collections.length === 0) {
    document.getElementById('hero-main-container').style.display = 'flex';
    document.getElementById('hero-grid').style.display = 'none';
    document.getElementById('empty-state').style.display = 'block';
    if (dock) dock.style.display = 'none';
  } else {
    document.getElementById('hero-main-container').style.display = 'flex';
    document.getElementById('hero-grid').style.display = 'grid';
    document.getElementById('empty-state').style.display = 'none';
    if (dock) dock.style.display = 'block';
    renderActiveCollection();
  }
}

async function refreshActiveCollectionComics() {
  if (!activeCollectionId) return;
  const col = collections.find(c => c.id === activeCollectionId);
  if (!col) return;

  const grid = document.getElementById('comics-catalog-grid');
  const emptyState = document.getElementById('collection-empty-comics');

  try {
    const rawComics = await invokeBackend('get_comics_by_collection', { collectionId: col.id });
    
    // Ordenamiento natural estricto A-Z por título
    activeCollectionComics = (rawComics || []).sort((a, b) => {
      return (a.title || '').localeCompare(b.title || '', 'es', { numeric: true, sensitivity: 'base' });
    });

    document.getElementById('cv-comics-count').textContent = activeCollectionComics.length;
    renderCollectionComicsGrid(activeCollectionComics);
  } catch (err) {
    grid.innerHTML = `<p style="color:#ef4444; font-size:14px; padding: 20px;">Error al cargar cómics: ${err}</p>`;
  }
}

async function showCollectionComicsView(collectionId) {
  closeAllModals();
  const col = collections.find(c => c.id === collectionId) || collections[currentCollectionIndex];
  if (!col) return;

  activeCollectionId = col.id;
  currentCollectionIndex = collections.findIndex(c => c.id === col.id);
  if (currentCollectionIndex < 0) currentCollectionIndex = 0;

  // Ocultar Hero Showcase y Dock inferior
  document.getElementById('hero-main-container').style.display = 'none';
  const dock = document.querySelector('.bottom-dock');
  if (dock) dock.style.display = 'none';

  // Mostrar vista de cómics
  const comicsView = document.getElementById('collection-comics-view');
  comicsView.style.display = 'block';

  // Configurar fondo de pantalla según la colección
  const bgSrc = col.background_image_base64 || toAssetUrl(col.background_image_path);
  if (bgSrc) {
    document.body.style.backgroundImage = `linear-gradient(rgba(0, 0, 0, 0.75), rgba(0, 0, 0, 0.90)), url('${bgSrc}')`;
  } else {
    document.body.style.backgroundImage = 'none';
    document.body.style.backgroundColor = '#000000';
  }

  // Actualizar datos de cabecera de la colección
  const rawProtagonist = col.protagonist || col.name;
  const protagonist = truncateText(rawProtagonist, 25).toUpperCase();
  document.getElementById('cv-protagonist').textContent = protagonist;
  document.getElementById('cv-title').textContent = col.name;
  document.getElementById('cv-description').textContent = truncateText(col.description, 600) || 'Esta colección reúne aventuras legendarias y ediciones especiales.';

  // Cargar cómics
  const grid = document.getElementById('comics-catalog-grid');
  grid.innerHTML = '<p style="color:#9ca3af; font-size:14px; padding: 20px;">Cargando cómics...</p>';
  document.getElementById('collection-empty-comics').style.display = 'none';

  try {
    const rawComics = await invokeBackend('get_comics_by_collection', { collectionId: col.id });
    
    // Ordenamiento natural estricto A-Z por título
    activeCollectionComics = (rawComics || []).sort((a, b) => {
      return (a.title || '').localeCompare(b.title || '', 'es', { numeric: true, sensitivity: 'base' });
    });

    document.getElementById('cv-comics-count').textContent = activeCollectionComics.length;
    renderCollectionComicsGrid(activeCollectionComics);
  } catch (err) {
    grid.innerHTML = `<p style="color:#ef4444; font-size:14px; padding: 20px;">Error al cargar cómics: ${err}</p>`;
    document.getElementById('cv-comics-count').textContent = '0';
  }
}

// RENDERIZADO DE LA GRILLA DE CÓMICS CON ANIMACIONES ESCALONADAS
function renderCollectionComicsGrid(comics) {
  const grid = document.getElementById('comics-catalog-grid');
  const emptyState = document.getElementById('collection-empty-comics');
  grid.innerHTML = '';

  if (!comics || comics.length === 0) {
    emptyState.style.display = 'block';
    return;
  }

  emptyState.style.display = 'none';

  comics.forEach((c, idx) => {
    const card = document.createElement('div');
    card.className = 'comic-card';
    card.setAttribute('data-comic-id', c.id);
    card.setAttribute('data-index', idx);
    // Retraso de animación en cascada suave
    card.style.animationDelay = `${Math.min(idx * 0.035, 0.45)}s`;

    // Formateo de título y número
    const displayTitle = truncateText(c.title, 40);
    const issueNum = c.issue_number ? `#${String(c.issue_number).padStart(2, '0')}` : `#${String(idx + 1).padStart(2, '0')}`;
    const pageCountText = `${c.page_count || 0} págs`;
    
    // Extensión / formato del archivo
    let fileExt = 'CBZ';
    if (c.file_path) {
      const ext = c.file_path.split('.').pop().toUpperCase();
      if (ext === 'CBR' || ext === 'CBZ' || ext === 'ZIP' || ext === 'RAR') {
        fileExt = ext;
      }
    }

    // Portada
    let coverHtml = '';
    if (c.cover_base64) {
      coverHtml = `<img src="${c.cover_base64}" alt="${c.title}" class="comic-card-cover-img" loading="lazy">`;
    } else {
      coverHtml = `
        <div class="comic-card-placeholder">
          <span class="comic-placeholder-icon">📖</span>
          <span class="comic-placeholder-format">${fileExt}</span>
        </div>
      `;
    }

    card.innerHTML = `
      <div class="comic-card-cover-box">
        ${coverHtml}
        <div class="comic-chips-overlay">
          <div class="comic-chip-top">
            <span class="comic-chip-issue">${issueNum}</span>
          </div>
          <div class="comic-chip-bottom">
            <span class="comic-chip-pages">${pageCountText}</span>
          </div>
        </div>
      </div>
      <div class="comic-card-footer">
        <span class="comic-card-title" title="${c.title}">${displayTitle}</span>
      </div>
    `;

    card.addEventListener('click', () => {
      playSound('click');
      openReader(c.id, c.title, idx);
    });

    grid.appendChild(card);
  });
}

// CARGA Y RENDERIZADO DE COLECCIONES EN EL SHOWCASE
async function loadCollections() {
  try {
    collections = await invokeBackend('get_collections');
  } catch (e) {
    collections = [];
  }

  updateCollectionsNavMenu();
  renderDock();

  const dock = document.querySelector('.bottom-dock');
  const comicsView = document.getElementById('collection-comics-view');

  // Solo alteramos visibilidad del Hero si la vista de cómics no está activa
  if (comicsView.style.display === 'none') {
    if (collections.length === 0) {
      document.body.style.backgroundImage = 'none';
      document.body.style.backgroundColor = '#000000';
      document.getElementById('hero-grid').style.display = 'none';
      document.getElementById('empty-state').style.display = 'block';
      if (dock) dock.style.display = 'none';
    } else {
      document.getElementById('hero-grid').style.display = 'grid';
      document.getElementById('empty-state').style.display = 'none';
      if (dock) dock.style.display = 'block';
      if (currentCollectionIndex >= collections.length) {
        currentCollectionIndex = 0;
      }
      renderActiveCollection();
    }
  }
}

// HELPER PARA TRUNCAR TEXTOS SEGÚN REGLAS DE NEGOCIO
function truncateText(str, maxLength) {
  if (!str) return '';
  const s = String(str).trim();
  return s.length > maxLength ? s.slice(0, maxLength) : s;
}

function renderActiveCollection() {
  if (collections.length === 0) return;
  const col = collections[currentCollectionIndex];

  // Actualizar Título de Protagonista (Bloque Rojo, máx 25 caracteres)
  const rawProtagonist = col.protagonist || col.name;
  const protagonist = truncateText(rawProtagonist, 25).toUpperCase();
  const mainTitle = document.getElementById('main-title');
  mainTitle.textContent = protagonist;
  mainTitle.setAttribute('data-text', protagonist);

  // Actualizar Sinopsis (máx 600 caracteres)
  const bioText = document.getElementById('bio-text');
  bioText.textContent = truncateText(col.description, 600) || 'Esta colección reúne aventuras legendarias y ediciones especiales.';

  // Actualizar Arte del Personaje
  const charImg = document.getElementById('character-main-img');
  const heroSrc = col.hero_image_base64 || toAssetUrl(col.hero_image_path);
  if (heroSrc) {
    charImg.src = heroSrc;
    charImg.style.display = 'block';
  } else {
    charImg.removeAttribute('src');
    charImg.style.display = 'none';
  }

  // Actualizar fondo si existe
  const bgSrc = col.background_image_base64 || toAssetUrl(col.background_image_path);
  if (bgSrc) {
    document.body.style.backgroundImage = `linear-gradient(rgba(0, 0, 0, 0.65), rgba(0, 0, 0, 0.85)), url('${bgSrc}')`;
  } else {
    document.body.style.backgroundImage = 'none';
    document.body.style.backgroundColor = '#000000';
  }

  // Resaltar tarjeta activa en el Dock
  document.querySelectorAll('.dock-card').forEach((card, idx) => {
    if (idx === currentCollectionIndex) {
      card.classList.add('active');
    } else {
      card.classList.remove('active');
    }
  });
}

// RENDERIZADO DEL DOCK INFERIOR CON SCROLL CONTINUO
function renderDock() {
  const dockTrack = document.getElementById('dock-track');
  dockTrack.innerHTML = '';

  collections.forEach((col, idx) => {
    const card = document.createElement('div');
    card.className = `dock-card ${idx === currentCollectionIndex ? 'active' : ''}`;
    card.setAttribute('data-index', idx);

    const protagonistName = truncateText(col.protagonist || col.name, 25);
    const thumbSrc = col.hero_image_base64 || toAssetUrl(col.hero_image_path);
    const thumbHtml = thumbSrc
      ? `<img src="${thumbSrc}" alt="${protagonistName}" class="dock-thumb-img">`
      : `<div class="dock-thumb-placeholder">📚</div>`;

    card.innerHTML = `
      <div class="dock-card-inner">
        <div class="dock-thumb-box">
          ${thumbHtml}
        </div>
        <div class="dock-label">
          <span>${protagonistName.toUpperCase()}</span>
        </div>
      </div>
    `;

    card.addEventListener('click', () => {
      playSound('click');
      currentCollectionIndex = idx;
      renderActiveCollection();
    });

    dockTrack.appendChild(card);
  });
}

function updateCollectionsNavMenu() {
  const menu = document.getElementById('nav-collections-menu');
  menu.innerHTML = `
    <li class="dropdown-header">Mis Colecciones (${collections.length})</li>
    <li class="divider"></li>
  `;

  collections.forEach((col, idx) => {
    const li = document.createElement('li');
    const a = document.createElement('a');
    a.href = '#';
    const heroTag = truncateText(col.protagonist || col.name, 25);
    a.textContent = `${col.name} (${heroTag})`;
    a.addEventListener('click', (e) => {
      e.preventDefault();
      playSound('click');
      currentCollectionIndex = idx;
      showCollectionComicsView(col.id);
    });
    li.appendChild(a);
    menu.appendChild(li);
  });

  const divider = document.createElement('li');
  divider.className = 'divider';
  menu.appendChild(divider);

  const newLi = document.createElement('li');
  newLi.innerHTML = `<a href="#" class="highlight-action">+ Nueva Colección</a>`;
  newLi.querySelector('a').addEventListener('click', (e) => {
    e.preventDefault();
    openCollectionFormModal(null);
  });
  menu.appendChild(newLi);
}

// MODAL FORMULARIO DE COLECCIÓN
function openCollectionFormModal(col) {
  playSound('click');
  const form = document.getElementById('collection-edit-form');
  form.reset();

  if (col) {
    document.getElementById('form-modal-title').textContent = 'EDITAR COLECCIÓN';
    document.getElementById('form-collection-id').value = col.id;
    document.getElementById('form-name').value = col.name;
    document.getElementById('form-protagonist').value = truncateText(col.protagonist || '', 25);
    document.getElementById('form-desc').value = truncateText(col.description || '', 600);
    document.getElementById('form-comics-path').value = col.comics_path || '';
    document.getElementById('form-bg-path').value = col.background_image_path || '';
    document.getElementById('form-hero-path').value = col.hero_image_path || '';
  } else {
    document.getElementById('form-modal-title').textContent = 'NUEVA COLECCIÓN';
    document.getElementById('form-collection-id').value = '0';
    document.getElementById('form-comics-path').value = '';
  }

  openModal('collection-form-modal');
}

async function handleSaveCollection() {
  const id = parseInt(document.getElementById('form-collection-id').value, 10);
  const name = document.getElementById('form-name').value.trim();
  const rawProtagonist = document.getElementById('form-protagonist').value.trim();
  const rawDesc = document.getElementById('form-desc').value.trim();
  const comicsPath = document.getElementById('form-comics-path').value.trim();
  const protagonist = truncateText(rawProtagonist, 25) || null;
  const description = truncateText(rawDesc, 600) || null;
  const bg = document.getElementById('form-bg-path').value.trim() || null;
  const hero = document.getElementById('form-hero-path').value.trim() || null;

  const btnSave = document.getElementById('btn-save-collection');
  const spinner = document.getElementById('btn-save-spinner');
  const btnText = document.getElementById('btn-save-text');

  if (btnSave) btnSave.disabled = true;
  if (spinner) spinner.style.display = 'inline-block';
  if (btnText) btnText.textContent = 'Guardando e indexando...';

  try {
    await invokeBackend('save_collection', {
      payload: {
        id: id > 0 ? id : null,
        name,
        protagonist,
        description,
        comics_path: comicsPath || null,
        background_image_path: bg,
        hero_image_path: hero,
        icon_data: null,
      }
    });

    closeAllModals();
    await loadCollections();
  } catch (err) {
    alert(`Error guardando colección: ${err}`);
  } finally {
    if (btnSave) btnSave.disabled = false;
    if (spinner) spinner.style.display = 'none';
    if (btnText) btnText.textContent = 'Guardar Colección';
  }
}

// MODAL QR Y SERVIDOR
async function openQrModal() {
  playSound('click');
  try {
    const status = await invokeBackend('get_server_status');
    document.getElementById('qr-url-text').textContent = status.url;
    const qrContainer = document.getElementById('qr-code-container');
    if (status.qr_base64) {
      qrContainer.innerHTML = status.qr_base64;
    }
  } catch (e) {
    // Servidor local
  }
  openModal('qr-modal');
}

// MODAL DISPOSITIVOS DE CONFIANZA
async function openDevicesModal() {
  playSound('click');
  const container = document.getElementById('devices-list-container');
  const countTag = document.getElementById('devices-count-tag');
  container.innerHTML = '<p style="color:#9ca3af; font-size:13px; padding: 12px 0;">Cargando dispositivos autorizados...</p>';
  openModal('devices-modal');

  try {
    const devices = await invokeBackend('get_trusted_devices');
    container.innerHTML = '';
    
    const count = (devices || []).length;
    if (countTag) {
      countTag.textContent = `${count} ${count === 1 ? 'ACTIVO' : 'ACTIVOS'}`;
    }

    if (!devices || devices.length === 0) {
      container.innerHTML = `
        <div class="devices-empty-state">
          <div class="empty-shield-icon">🛡️</div>
          <div class="empty-shield-title">NO HAY DISPOSITIVOS REGISTRADOS</div>
          <div class="empty-shield-desc">Ingresa un nombre arriba para autorizar un dispositivo o comparte el código QR.</div>
        </div>
      `;
    } else {
      devices.forEach(d => {
        const div = document.createElement('div');
        div.className = 'device-card';

        // Determinar icono según el nombre
        const lowerName = (d.device_name || '').toLowerCase();
        let icon = '📱';
        if (lowerName.includes('pc') || lowerName.includes('mac') || lowerName.includes('laptop') || lowerName.includes('desktop') || lowerName.includes('computador')) {
          icon = '💻';
        } else if (lowerName.includes('ipad') || lowerName.includes('tablet') || lowerName.includes('tab')) {
          icon = '📟';
        }

        const tokenSnippet = d.token ? `${d.token.substring(0, 10)}...` : '******';
        const dateStr = d.created_at || 'Reciente';

        div.innerHTML = `
          <div class="device-info-left">
            <div class="device-avatar">${icon}</div>
            <div class="device-details">
              <div class="device-name">${d.device_name}</div>
              <div class="device-meta">
                <span class="device-token-badge" title="Token de autorización">🔑 ${tokenSnippet}</span>
                <span>📅 ${dateStr}</span>
                <span class="device-status-chip">● Activo</span>
              </div>
            </div>
          </div>
          <button class="btn-revoke-device" data-id="${d.id}" title="Revocar autorización a este dispositivo">
            ✕ Revocar
          </button>
        `;

        div.querySelector('.btn-revoke-device').addEventListener('click', async () => {
          if (confirm(`¿Deseas revocar la autorización para '${d.device_name}'?`)) {
            try {
              await invokeBackend('remove_trusted_device', { id: d.id });
              await openDevicesModal();
            } catch (err) {
              alert(`Error al eliminar dispositivo: ${err}`);
            }
          }
        });

        container.appendChild(div);
      });
    }
  } catch (e) {
    container.innerHTML = `<p style="color:#ef4444; font-size:13px; padding: 12px 0;">Error al cargar dispositivos: ${e}</p>`;
  }
}

// VISOR DE LECTURA (READER) Y CONTINUIDAD DE TOMOS CON TRANSICIÓN SUAVE
async function openReader(comicId, title, comicIndex = -1) {
  currentReadingComicId = comicId;
  currentReadingComicIndex = comicIndex;
  currentReadingPage = 0;
  
  const finishOverlay = document.getElementById('reader-finish-overlay');
  if (finishOverlay) finishOverlay.classList.remove('active');

  resetReaderZoom();
  document.getElementById('reader-comic-title').textContent = title;
  const reader = document.getElementById('comic-reader');
  if (reader) reader.classList.add('active');

  await loadReaderPage();
}

function closeReader() {
  resetReaderZoom();
  const reader = document.getElementById('comic-reader');
  const finishOverlay = document.getElementById('reader-finish-overlay');
  if (reader) reader.classList.remove('active');
  if (finishOverlay) finishOverlay.classList.remove('active');
}

function openNextComicInReader() {
  if (currentReadingComicIndex >= 0 && currentReadingComicIndex + 1 < activeCollectionComics.length) {
    const nextIndex = currentReadingComicIndex + 1;
    const nextComic = activeCollectionComics[nextIndex];
    openReader(nextComic.id, nextComic.title, nextIndex);
  } else {
    closeReader();
  }
}

async function loadReaderPage() {
  if (!currentReadingComicId) return;
  const pageImg = document.getElementById('reader-page-img');
  if (pageImg) pageImg.classList.add('page-loading');

  try {
    const pageData = await invokeBackend('get_comic_page', {
      comicId: currentReadingComicId,
      pageIndex: currentReadingPage,
    });
    currentReadingTotalPages = pageData.total_pages;

    if (pageImg) {
      pageImg.src = pageData.image_data_base64;
      pageImg.classList.remove('page-loading');
    }

    document.getElementById('reader-current-page').textContent = currentReadingPage + 1;
    document.getElementById('reader-total-pages').textContent = currentReadingTotalPages;
  } catch (err) {
    if (pageImg) pageImg.classList.remove('page-loading');
    alert(`Error al cargar página: ${err}`);
  }
}

async function changeReaderPage(delta) {
  const newPage = currentReadingPage + delta;
  
  if (newPage >= currentReadingTotalPages) {
    showReaderFinishOverlay();
    return;
  }
  
  if (newPage >= 0 && newPage < currentReadingTotalPages) {
    const finishOverlay = document.getElementById('reader-finish-overlay');
    if (finishOverlay) finishOverlay.classList.remove('active');

    resetReaderZoom();
    currentReadingPage = newPage;
    await loadReaderPage();
  }
}

function showReaderFinishOverlay() {
  const overlay = document.getElementById('reader-finish-overlay');
  const comicName = document.getElementById('finish-current-comic-name');
  const btnNext = document.getElementById('btn-reader-next-comic');
  
  let currentTitle = 'Este cómic';
  if (currentReadingComicIndex >= 0 && currentReadingComicIndex < activeCollectionComics.length) {
    currentTitle = activeCollectionComics[currentReadingComicIndex].title;
  }
  comicName.textContent = currentTitle;

  // Si existe un siguiente cómic en la lista activa, configurar el botón
  if (currentReadingComicIndex >= 0 && currentReadingComicIndex + 1 < activeCollectionComics.length) {
    const nextComic = activeCollectionComics[currentReadingComicIndex + 1];
    btnNext.style.display = 'block';
    btnNext.textContent = `Siguiente Tomo ▶ (${truncateText(nextComic.title, 25)})`;
  } else {
    btnNext.style.display = 'none';
  }

  if (overlay) overlay.classList.add('active');
}

async function loadServerStatus() {
  try {
    const status = await invokeBackend('get_server_status');
    const label = document.getElementById('nav-server-label');
    if (label) {
      label.textContent = `Servidor Activo (${status.port})`;
    }
  } catch (e) {
    // Servidor local
  }
}

// GLITCH CANVAS EFECTO
function setupGlitchCanvas() {
  const canvas = document.getElementById('glitch-canvas');
  if (!canvas) return;
  const ctx = canvas.getContext('2d');
  let width = (canvas.width = window.innerWidth);
  let height = (canvas.height = window.innerHeight);

  window.addEventListener('resize', () => {
    width = canvas.width = window.innerWidth;
    height = canvas.height = window.innerHeight;
  });

  const particles = [];
  for (let i = 0; i < 35; i++) {
    particles.push({
      x: Math.random() * width,
      y: Math.random() * height,
      size: Math.random() * 2 + 1,
      speedX: (Math.random() - 0.5) * 0.8,
      speedY: (Math.random() - 0.5) * 0.8,
      color: Math.random() > 0.5 ? '#ff0038' : '#00f0ff',
      alpha: Math.random() * 0.5 + 0.2
    });
  }

  function animate() {
    ctx.clearRect(0, 0, width, height);
    particles.forEach(p => {
      p.x += p.speedX;
      p.y += p.speedY;
      if (p.x < 0) p.x = width;
      if (p.x > width) p.x = 0;
      if (p.y < 0) p.y = height;
      if (p.y > height) p.y = 0;

      ctx.fillStyle = p.color;
      ctx.globalAlpha = p.alpha;
      ctx.fillRect(p.x, p.y, p.size, p.size);
    });
    requestAnimationFrame(animate);
  }
  animate();
}

// MOCK BACKEND PARA PRUEBAS WEB
function mockBackend(command, args) {
  if (command === 'get_collections') {
    return [
      {
        id: 1,
        name: 'Spider-Man: Into the Spider-Verse',
        protagonist: 'SPIDER-MAN',
        description: 'Miles Morales se adentra en el multiverso para detener una amenaza interdimensional que pondrá en riesgo toda la realidad.',
        hero_image_path: 'assets/miles_hero.png',
        background_image_path: 'assets/city_bg.jpg',
        created_at: '2026-09-22',
      }
    ];
  }
  if (command === 'get_server_status') {
    return {
      running: true,
      port: 8080,
      local_ip: '127.0.0.1',
      url: 'http://127.0.0.1:8080',
      qr_base64: '<svg width="200" height="200" xmlns="http://www.w3.org/2000/svg"><rect width="200" height="200" fill="#000"/><text x="100" y="100" fill="#fff" text-anchor="middle">QR CODE</text></svg>',
    };
  }
  if (command === 'get_trusted_devices') {
    return [];
  }
  if (command === 'get_comics_by_collection') {
    return [
      { id: 101, collection_id: 1, title: 'Spider-Verse #01: Origins', file_path: 'test1.cbz', page_count: 32, has_cover: false, cover_base64: null, issue_number: 1 },
      { id: 102, collection_id: 1, title: 'Spider-Verse #02: Web Warriors', file_path: 'test2.cbz', page_count: 28, has_cover: false, cover_base64: null, issue_number: 2 },
      { id: 103, collection_id: 1, title: 'Spider-Verse #03: Final Stand', file_path: 'test3.cbz', page_count: 40, has_cover: false, cover_base64: null, issue_number: 3 }
    ];
  }
  if (command === 'get_comic_page') {
    return {
      comic_id: args.comicId,
      page_index: args.pageIndex,
      total_pages: 3,
      image_data_base64: 'data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="600" height="900"><rect width="100%" height="100%" fill="%23111827"/><text x="50%" y="50%" fill="%23fff" font-size="28" text-anchor="middle">Página ' + (args.pageIndex + 1) + '</text></svg>',
      width: 600,
      height: 900,
    };
  }
  return null;
}
