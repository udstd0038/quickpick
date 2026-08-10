export type UiLanguage =
  | "system"
  | "zh-Hans"
  | "zh-Hant"
  | "en"
  | "ko"
  | "ja"
  | "fr"
  | "de"
  | "es";

export type TranslationLanguage = Exclude<UiLanguage, "system">;

export const uiLanguageOptions: Array<{ id: UiLanguage; label: string }> = [
  { id: "system", label: "系统语言" },
  { id: "zh-Hans", label: "简体中文" },
  { id: "zh-Hant", label: "繁体中文" },
  { id: "en", label: "英文" },
  { id: "ko", label: "韩文" },
  { id: "ja", label: "日文" },
  { id: "fr", label: "法文" },
  { id: "de", label: "德文" },
  { id: "es", label: "西班牙文" },
];

export const translationLanguageOptions: Array<{
  id: TranslationLanguage;
  label: string;
}> = [
  { id: "zh-Hans", label: "简体中文" },
  { id: "zh-Hant", label: "繁体中文" },
  { id: "en", label: "英文" },
  { id: "ko", label: "韩文" },
  { id: "ja", label: "日文" },
  { id: "fr", label: "法文" },
  { id: "de", label: "德文" },
  { id: "es", label: "西班牙文" },
];

export function systemLanguage(): TranslationLanguage {
  const language = (navigator.language || "").toLowerCase();
  if (language.startsWith("zh")) {
    return /zh-(tw|hk|mo)/i.test(navigator.language)
      ? "zh-Hant"
      : "zh-Hans";
  }
  if (language.startsWith("en")) {
    return "en";
  }
  if (language.startsWith("ko")) {
    return "ko";
  }
  if (language.startsWith("ja")) {
    return "ja";
  }
  if (language.startsWith("fr")) {
    return "fr";
  }
  if (language.startsWith("de")) {
    return "de";
  }
  if (language.startsWith("es")) {
    return "es";
  }
  return "zh-Hans";
}

export function resolveUiLanguage(value: UiLanguage): TranslationLanguage {
  return value === "system" ? systemLanguage() : value;
}

export function targetLanguageForUiLanguage(value: UiLanguage): TranslationLanguage {
  return resolveUiLanguage(value);
}

type LocaleStrings = Record<TranslationLanguage, string>;

const entry = (
  zhHant: string,
  en: string,
  ko: string,
  ja: string,
  fr: string,
  de: string,
  es: string,
): LocaleStrings => ({
  "zh-Hans": "",
  "zh-Hant": zhHant,
  en,
  ko,
  ja,
  fr,
  de,
  es,
});

const translations: Record<string, LocaleStrings> = {
  系统语言: entry("系統語言", "System Language", "시스템 언어", "システム言語", "Langue du système", "Systemsprache", "Idioma del sistema"),
  简体中文: entry("簡體中文", "Simplified Chinese", "중국어 간체", "簡体中文", "Chinois simplifié", "Vereinfachtes Chinesisch", "Chino simplificado"),
  繁体中文: entry("繁體中文", "Traditional Chinese", "중국어 번체", "繁体中文", "Chinois traditionnel", "Traditionelles Chinesisch", "Chino tradicional"),
  英文: entry("英文", "English", "영어", "英語", "Anglais", "Englisch", "Inglés"),
  英语: entry("英語", "English", "영어", "英語", "Anglais", "Englisch", "Inglés"),
  韩文: entry("韓文", "Korean", "한국어", "韓国語", "Coréen", "Koreanisch", "Coreano"),
  韩语: entry("韓語", "Korean", "한국어", "韓国語", "Coréen", "Koreanisch", "Coreano"),
  日文: entry("日文", "Japanese", "일본어", "日本語", "Japonais", "Japanisch", "Japonés"),
  日语: entry("日語", "Japanese", "일본어", "日本語", "Japonais", "Japanisch", "Japonés"),
  法文: entry("法文", "French", "프랑스어", "フランス語", "Français", "Französisch", "Francés"),
  法语: entry("法語", "French", "프랑스어", "フランス語", "Français", "Französisch", "Francés"),
  德文: entry("德文", "German", "독일어", "ドイツ語", "Allemand", "Deutsch", "Alemán"),
  德语: entry("德語", "German", "독일어", "ドイツ語", "Allemand", "Deutsch", "Alemán"),
  西班牙文: entry("西班牙文", "Spanish", "스페인어", "スペイン語", "Espagnol", "Spanisch", "Español"),
  西班牙语: entry("西班牙語", "Spanish", "스페인어", "スペイン語", "Espagnol", "Spanisch", "Español"),
  自动检测: entry("自動偵測", "Auto Detect", "자동 감지", "自動検出", "Détection automatique", "Automatisch erkennen", "Detección automática"),
  语言: entry("語言", "Language", "언어", "言語", "Langue", "Sprache", "Idioma"),
  通用: entry("通用", "General", "일반", "一般", "Général", "Allgemein", "General"),
  快捷键: entry("快捷鍵", "Hotkeys", "단축키", "ショートカット", "Raccourcis", "Tastenkürzel", "Atajos"),
  划词模型: entry("劃詞模型", "Selection Model", "선택 모델", "選択モデル", "Modèle de sélection", "Auswahlmodell", "Modelo de selección"),
  截图模型: entry("截圖模型", "Screenshot Model", "스크린샷 모델", "スクリーンショットモデル", "Modèle de capture", "Screenshot-Modell", "Modelo de captura"),
  输入模型: entry("輸入模型", "Input Model", "입력 모델", "入力モデル", "Modèle de saisie", "Eingabemodell", "Modelo de entrada"),
  外观: entry("外觀", "Appearance", "모양", "外観", "Apparence", "Darstellung", "Apariencia"),
  隐私: entry("隱私", "Privacy", "개인정보", "プライバシー", "Confidentialité", "Datenschutz", "Privacidad"),
  开机自启: entry("開機自啟", "Start on Boot", "부팅 시 시작", "起動時に開始", "Démarrage automatique", "Beim Start ausführen", "Iniciar con el sistema"),
  托盘常驻: entry("托盤常駐", "Tray Resident", "트레이 상주", "トレイ常駐", "Résident dans la barre", "Im Tray bleiben", "Residente en bandeja"),
  "AI 请求超时": entry("AI 請求逾時", "AI Request Timeout", "AI 요청 제한 시간", "AIリクエストのタイムアウト", "Délai de requête IA", "KI-Anfrage-Timeout", "Tiempo de espera de IA"),
  已启用: entry("已啟用", "Enabled", "사용 중", "有効", "Activé", "Aktiviert", "Activado"),
  已关闭: entry("已關閉", "Disabled", "사용 안 함", "無効", "Désactivé", "Deaktiviert", "Desactivado"),
  划词菜单: entry("劃詞選單", "Selection Menu", "선택 메뉴", "選択メニュー", "Menu de sélection", "Auswahlmenü", "Menú de selección"),
  区域截图: entry("區域截圖", "Region Screenshot", "영역 캡처", "領域スクリーンショット", "Capture de région", "Bereichs-Screenshot", "Captura de región"),
  输入翻译: entry("輸入翻譯", "Input Translation", "입력 번역", "入力翻訳", "Traduction de saisie", "Eingabeübersetzung", "Traducción de entrada"),
  设置: entry("設定", "Settings", "설정", "設定", "Paramètres", "Einstellungen", "Configuración"),
  设置分类: entry("設定分類", "Settings Categories", "설정 분류", "設定カテゴリ", "Catégories de paramètres", "Einstellungskategorien", "Categorías de configuración"),
  保存设置: entry("儲存設定", "Save Settings", "설정 저장", "設定を保存", "Enregistrer les paramètres", "Einstellungen speichern", "Guardar configuración"),
  设置已保存: entry("設定已儲存", "Settings Saved", "설정이 저장되었습니다", "設定を保存しました", "Paramètres enregistrés", "Einstellungen gespeichert", "Configuración guardada"),
  设置已载入: entry("設定已載入", "Settings Loaded", "설정을 불러왔습니다", "設定を読み込みました", "Paramètres chargés", "Einstellungen geladen", "Configuración cargada"),
  "读取设置失败，已使用默认值": entry("讀取設定失敗，已使用預設值", "Failed to load settings, defaults used", "설정을 불러오지 못해 기본값을 사용합니다", "設定の読み込みに失敗、既定値を使用", "Échec du chargement, valeurs par défaut", "Einstellungen konnten nicht geladen werden", "No se pudieron cargar los ajustes"),
  有未保存的修改: entry("有未儲存的修改", "Unsaved Changes", "저장되지 않은 변경 사항", "未保存の変更", "Modifications non enregistrées", "Ungespeicherte Änderungen", "Cambios sin guardar"),
  正在保存: entry("正在儲存", "Saving", "저장 중", "保存中", "Enregistrement", "Wird gespeichert", "Guardando"),
  保存设置失败: entry("儲存設定失敗", "Failed to Save Settings", "설정 저장 실패", "設定の保存に失敗", "Échec de l'enregistrement", "Speichern fehlgeschlagen", "No se pudo guardar"),
  主题: entry("主題", "Theme", "테마", "テーマ", "Thème", "Design", "Tema"),
  跟随系统: entry("跟隨系統", "Follow System", "시스템 따르기", "システムに従う", "Suivre le système", "System folgen", "Seguir sistema"),
  浅色: entry("淺色", "Light", "라이트", "ライト", "Clair", "Hell", "Claro"),
  深色: entry("深色", "Dark", "다크", "ダーク", "Sombre", "Dunkel", "Oscuro"),
  窗口效果: entry("視窗效果", "Window Effect", "창 효과", "ウィンドウ効果", "Effet de fenêtre", "Fenstereffekt", "Efecto de ventana"),
  不透明度: entry("不透明度", "Opacity", "불투명도", "不透明度", "Opacité", "Deckkraft", "Opacidad"),
  文本供应商: entry("文字供應商", "Text Provider", "텍스트 공급자", "テキストプロバイダー", "Fournisseur de texte", "Textanbieter", "Proveedor de texto"),
  "文本 Base URL": entry("文字 Base URL", "Text Base URL", "텍스트 Base URL", "テキストBase URL", "URL de base du texte", "Text-Basis-URL", "URL base de texto"),
  文本模型: entry("文字模型", "Text Model", "텍스트 모델", "テキストモデル", "Modèle de texte", "Textmodell", "Modelo de texto"),
  "文本 API Key": entry("文字 API Key", "Text API Key", "텍스트 API 키", "テキストAPIキー", "Clé API de texte", "Text-API-Schlüssel", "Clave API de texto"),
  "文本 Key 状态": entry("文字 Key 狀態", "Text Key Status", "텍스트 키 상태", "テキストキー状態", "État de la clé de texte", "Text-Key-Status", "Estado de clave de texto"),
  已加密保存: entry("已加密儲存", "Encrypted", "암호화 저장됨", "暗号化して保存", "Chiffré", "Verschlüsselt", "Cifrado"),
  未配置: entry("未配置", "Not Configured", "설정되지 않음", "未設定", "Non configuré", "Nicht konfiguriert", "No configurado"),
  "保存文本 Key": entry("儲存文字 Key", "Save Text Key", "텍스트 키 저장", "テキストキーを保存", "Enregistrer la clé de texte", "Text-Key speichern", "Guardar clave de texto"),
  视觉供应商: entry("視覺供應商", "Vision Provider", "비전 공급자", "ビジョンプロバイダー", "Fournisseur visuel", "Bildanbieter", "Proveedor visual"),
  "视觉 Base URL": entry("視覺 Base URL", "Vision Base URL", "비전 Base URL", "ビジョンBase URL", "URL de base visuelle", "Bild-Basis-URL", "URL base visual"),
  视觉模型: entry("視覺模型", "Vision Model", "비전 모델", "ビジョンモデル", "Modèle visuel", "Bildmodell", "Modelo visual"),
  "视觉 API Key": entry("視覺 API Key", "Vision API Key", "비전 API 키", "ビジョンAPIキー", "Clé API visuelle", "Bild-API-Schlüssel", "Clave API visual"),
  "视觉 Key 状态": entry("視覺 Key 狀態", "Vision Key Status", "비전 키 상태", "ビジョンキー状態", "État de la clé visuelle", "Bild-Key-Status", "Estado de clave visual"),
  "保存视觉 Key": entry("儲存視覺 Key", "Save Vision Key", "비전 키 저장", "ビジョンキーを保存", "Enregistrer la clé visuelle", "Bild-Key speichern", "Guardar clave visual"),
  输入供应商: entry("輸入供應商", "Input Provider", "입력 공급자", "入力プロバイダー", "Fournisseur de saisie", "Eingabeanbieter", "Proveedor de entrada"),
  "输入 Base URL": entry("輸入 Base URL", "Input Base URL", "입력 Base URL", "入力Base URL", "URL de base de saisie", "Eingabe-Basis-URL", "URL base de entrada"),
  "输入 API Key": entry("輸入 API Key", "Input API Key", "입력 API 키", "入力APIキー", "Clé API de saisie", "Eingabe-API-Schlüssel", "Clave API de entrada"),
  "输入 Key 状态": entry("輸入 Key 狀態", "Input Key Status", "입력 키 상태", "入力キー状態", "État de la clé de saisie", "Eingabe-Key-Status", "Estado de clave de entrada"),
  "保存输入 Key": entry("儲存輸入 Key", "Save Input Key", "입력 키 저장", "入力キーを保存", "Enregistrer la clé de saisie", "Eingabe-Key speichern", "Guardar clave de entrada"),
  清除: entry("清除", "Clear", "지우기", "クリア", "Effacer", "Löschen", "Borrar"),
  复制: entry("複製", "Copy", "복사", "コピー", "Copier", "Kopieren", "Copiar"),
  翻译: entry("翻譯", "Translate", "번역", "翻訳", "Traduire", "Übersetzen", "Traducir"),
  总结: entry("總結", "Summarize", "요약", "要約", "Résumer", "Zusammenfassen", "Resumir"),
  搜索: entry("搜尋", "Search", "검색", "検索", "Rechercher", "Suchen", "Buscar"),
  提取: entry("提取", "Extract", "추출", "抽出", "Extraire", "Extrahieren", "Extraer"),
  固定: entry("固定", "Pin", "고정", "固定", "Épingler", "Anpinnen", "Fijar"),
  取消固定: entry("取消固定", "Unpin", "고정 해제", "固定解除", "Détacher", "Lösen", "Desfijar"),
  关闭: entry("關閉", "Close", "닫기", "閉じる", "Fermer", "Schließen", "Cerrar"),
  源语言: entry("來源語言", "Source Language", "원본 언어", "ソース言語", "Langue source", "Quellsprache", "Idioma de origen"),
  目标语言: entry("目標語言", "Target Language", "대상 언어", "ターゲット言語", "Langue cible", "Zielsprache", "Idioma de destino"),
  切换翻译方向: entry("切換翻譯方向", "Switch Translation Direction", "번역 방향 전환", "翻訳方向を切り替え", "Inverser la direction", "Übersetzungsrichtung wechseln", "Cambiar dirección"),
  输入要翻译的文本: entry("輸入要翻譯的文字", "Enter text to translate", "번역할 텍스트 입력", "翻訳するテキストを入力", "Saisissez le texte à traduire", "Zu übersetzenden Text eingeben", "Introduce texto para traducir"),
  翻译中: entry("翻譯中", "Translating", "번역 중", "翻訳中", "Traduction en cours", "Übersetze", "Traduciendo"),
  "正在处理...": entry("正在處理...", "Processing...", "처리 중...", "処理中...", "Traitement...", "Verarbeitung...", "Procesando..."),
  翻译请求失败: entry("翻譯請求失敗", "Translation Request Failed", "번역 요청 실패", "翻訳リクエストに失敗", "Échec de la demande de traduction", "Übersetzungsanfrage fehlgeschlagen", "Error en la solicitud de traducción"),
  切换语言失败: entry("切換語言失敗", "Language Switch Failed", "언어 전환 실패", "言語切替に失敗", "Échec du changement de langue", "Sprachwechsel fehlgeschlagen", "Error al cambiar idioma"),
  暂无结果: entry("暫無結果", "No Results", "결과 없음", "結果はありません", "Aucun résultat", "Keine Ergebnisse", "Sin resultados"),
  待处理: entry("待處理", "Pending", "대기 중", "待機中", "En attente", "Ausstehend", "Pendiente"),
  正在处理: entry("正在處理", "Processing", "처리 중", "処理中", "Traitement", "Verarbeitung", "Procesando"),
  翻译结果: entry("翻譯結果", "Translation Result", "번역 결과", "翻訳結果", "Résultat de traduction", "Übersetzungsergebnis", "Resultado de traducción"),
  处理失败: entry("處理失敗", "Failed", "처리 실패", "処理に失敗", "Échec", "Fehlgeschlagen", "Error"),
  复制结果: entry("複製結果", "Copy Result", "결과 복사", "結果をコピー", "Copier le résultat", "Ergebnis kopieren", "Copiar resultado"),
  重试: entry("重試", "Retry", "다시 시도", "再試行", "Réessayer", "Erneut versuchen", "Reintentar"),
  "正在读取屏幕截图...": entry("正在讀取螢幕截圖...", "Reading screenshot...", "스크린샷 읽는 중...", "スクリーンショットを読み込み中...", "Lecture de la capture...", "Screenshot wird geladen...", "Leyendo captura..."),
  当前屏幕: entry("目前螢幕", "Current Screen", "현재 화면", "現在の画面", "Écran actuel", "Aktueller Bildschirm", "Pantalla actual"),
  等待截图区域: entry("等待截圖區域", "Waiting for Screenshot Region", "캡처 영역 대기 중", "キャプチャ領域を待機中", "En attente de la région", "Warte auf Screenshot-Bereich", "Esperando región de captura"),
  正在检查核心连接: entry("正在檢查核心連線", "Checking Core Connection", "코어 연결 확인 중", "コア接続を確認中", "Vérification de la connexion", "Prüfe Kernverbindung", "Comprobando conexión"),
  前端预览模式: entry("前端預覽模式", "Frontend Preview Mode", "프런트엔드 미리보기 모드", "フロントエンドプレビューモード", "Mode aperçu frontend", "Frontend-Vorschau", "Modo de vista previa"),
  "QuickPick 设置": entry("QuickPick 設定", "QuickPick Settings", "QuickPick 설정", "QuickPick 設定", "Paramètres QuickPick", "QuickPick-Einstellungen", "Configuración de QuickPick"),
  "QuickPick 结果": entry("QuickPick 結果", "QuickPick Result", "QuickPick 결과", "QuickPick 結果", "Résultat QuickPick", "QuickPick-Ergebnis", "Resultado de QuickPick"),
  "QuickPick 划词": entry("QuickPick 劃詞", "QuickPick Selection", "QuickPick 선택", "QuickPick 選択", "Sélection QuickPick", "QuickPick-Auswahl", "Selección de QuickPick"),
  最小化: entry("最小化", "Minimize", "최소화", "最小化", "Réduire", "Minimieren", "Minimizar"),
  最大化或还原: entry("最大化或還原", "Maximize or Restore", "최대화 또는 복원", "最大化または元に戻す", "Agrandir ou restaurer", "Maximieren oder Wiederherstellen", "Maximizar o restaurar"),
  关闭设置: entry("關閉設定", "Close Settings", "설정 닫기", "設定を閉じる", "Fermer les paramètres", "Einstellungen schließen", "Cerrar configuración"),
  内容历史: entry("內容歷史", "Content History", "콘텐츠 기록", "コンテンツ履歴", "Historique du contenu", "Inhaltsverlauf", "Historial de contenido"),
  "默认不保存文本、截图和 AI 结果": entry("預設不儲存文字、截圖和 AI 結果", "Text, screenshots, and AI results are not saved by default", "기본적으로 텍스트, 스크린샷, AI 결과를 저장하지 않습니다", "既定ではテキスト、スクリーンショット、AI結果を保存しません", "Le texte, les captures et les résultats IA ne sont pas enregistrés", "Text, Screenshots und KI-Ergebnisse werden standardmäßig nicht gespeichert", "El texto, las capturas y los resultados de IA no se guardan de forma predeterminada"),
  "API Key": entry("API Key", "API Key", "API 키", "APIキー", "Clé API", "API-Schlüssel", "Clave API"),
  "使用 Windows DPAPI 本机加密保存": entry("使用 Windows DPAPI 本機加密儲存", "Stored encrypted locally with Windows DPAPI", "Windows DPAPI로 로컬 암호화 저장", "Windows DPAPIでローカル暗号化保存", "Chiffré localement avec Windows DPAPI", "Lokal mit Windows DPAPI verschlüsselt", "Cifrado localmente con Windows DPAPI"),
  供应商: entry("供應商", "Provider", "공급자", "プロバイダー", "Fournisseur", "Anbieter", "Proveedor"),
  "AI 配置": entry("AI 配置", "AI Configuration", "AI 설정", "AI構成", "Configuration IA", "KI-Konfiguration", "Configuración de IA"),
  "AI 配置已保存": entry("AI 配置已儲存", "AI Configuration Saved", "AI 설정 저장됨", "AI構成を保存しました", "Configuration IA enregistrée", "KI-Konfiguration gespeichert", "Configuración de IA guardada"),
  "AI 配置已载入": entry("AI 配置已載入", "AI Configuration Loaded", "AI 설정 불러옴", "AI構成を読み込みました", "Configuration IA chargée", "KI-Konfiguration geladen", "Configuración de IA cargada"),
  "API Key 不能为空": entry("API Key 不能為空", "API Key cannot be empty", "API 키는 비어 있을 수 없습니다", "APIキーは空にできません", "La clé API ne peut pas être vide", "API-Schlüssel darf nicht leer sein", "La clave API no puede estar vacía"),
  "Key 已保存": entry("Key 已儲存", "Key Saved", "키 저장됨", "キーを保存しました", "Clé enregistrée", "Schlüssel gespeichert", "Clave guardada"),
  "Key 未配置": entry("Key 未配置", "Key Not Configured", "키 미설정", "キー未設定", "Clé non configurée", "Schlüssel nicht konfiguriert", "Clave no configurada"),
  "正在检查文本模型 Key": entry("正在檢查文字模型 Key", "Checking Text Model Key", "텍스트 모델 키 확인 중", "テキストモデルキーを確認中", "Vérification de la clé de texte", "Prüfe Textmodell-Schlüssel", "Comprobando clave de texto"),
  "正在检查视觉模型 Key": entry("正在檢查視覺模型 Key", "Checking Vision Model Key", "비전 모델 키 확인 중", "ビジョンモデルキーを確認中", "Vérification de la clé visuelle", "Prüfe Bildmodell-Schlüssel", "Comprobando clave visual"),
  "正在检查输入模型 Key": entry("正在檢查輸入模型 Key", "Checking Input Model Key", "입력 모델 키 확인 중", "入力モデルキーを確認中", "Vérification de la clé de saisie", "Prüfe Eingabemodell-Schlüssel", "Comprobando clave de entrada"),
  "文本模型 Key 已加密保存": entry("文字模型 Key 已加密儲存", "Text Model Key Encrypted", "텍스트 모델 키 암호화 저장", "テキストモデルキーを暗号化保存", "Clé de texte chiffrée", "Textmodell-Schlüssel verschlüsselt", "Clave de texto cifrada"),
  "文本模型 Key 未配置": entry("文字模型 Key 未配置", "Text Model Key Not Configured", "텍스트 모델 키 미설정", "テキストモデルキー未設定", "Clé de texte non configurée", "Textmodell-Schlüssel nicht konfiguriert", "Clave de texto no configurada"),
  "视觉模型 Key 已加密保存": entry("視覺模型 Key 已加密儲存", "Vision Model Key Encrypted", "비전 모델 키 암호화 저장", "ビジョンモデルキーを暗号化保存", "Clé visuelle chiffrée", "Bildmodell-Schlüssel verschlüsselt", "Clave visual cifrada"),
  "视觉模型 Key 未配置": entry("視覺模型 Key 未配置", "Vision Model Key Not Configured", "비전 모델 키 미설정", "ビジョンモデルキー未設定", "Clé visuelle non configurée", "Bildmodell-Schlüssel nicht konfiguriert", "Clave visual no configurada"),
  "输入模型 Key 已加密保存": entry("輸入模型 Key 已加密儲存", "Input Model Key Encrypted", "입력 모델 키 암호화 저장", "入力モデルキーを暗号化保存", "Clé de saisie chiffrée", "Eingabemodell-Schlüssel verschlüsselt", "Clave de entrada cifrada"),
  "输入模型 Key 未配置": entry("輸入模型 Key 未配置", "Input Model Key Not Configured", "입력 모델 키 미설정", "入力モデルキー未設定", "Clé de saisie non configurée", "Eingabemodell-Schlüssel nicht konfiguriert", "Clave de entrada no configurada"),
  选择外观主题: entry("選擇外觀主題", "Select Appearance Theme", "테마 선택", "テーマを選択", "Choisir le thème", "Design auswählen", "Seleccionar tema"),
  选择窗口效果: entry("選擇視窗效果", "Select Window Effect", "창 효과 선택", "ウィンドウ効果を選択", "Choisir l'effet", "Fenstereffekt auswählen", "Seleccionar efecto"),
  "选择 AI 供应商": entry("選擇 AI 供應商", "Select AI Provider", "AI 공급자 선택", "AIプロバイダーを選択", "Choisir le fournisseur IA", "KI-Anbieter auswählen", "Seleccionar proveedor de IA"),
  选择文本模型供应商: entry("選擇文字模型供應商", "Select Text Model Provider", "텍스트 모델 공급자 선택", "テキストモデルプロバイダーを選択", "Choisir le fournisseur de texte", "Textanbieter auswählen", "Seleccionar proveedor de texto"),
  选择视觉模型供应商: entry("選擇視覺模型供應商", "Select Vision Model Provider", "비전 모델 공급자 선택", "ビジョンモデルプロバイダーを選択", "Choisir le fournisseur visuel", "Bildanbieter auswählen", "Seleccionar proveedor visual"),
  选择输入模型供应商: entry("選擇輸入模型供應商", "Select Input Model Provider", "입력 모델 공급자 선택", "入力モデルプロバイダーを選択", "Choisir le fournisseur de saisie", "Eingabeanbieter auswählen", "Seleccionar proveedor de entrada"),
  按下快捷键组合: entry("按下快捷鍵組合", "Press a Hotkey", "단축키를 누르세요", "ショートカットキーを押してください", "Appuyez sur un raccourci", "Tastenkürzel drücken", "Pulsa un atajo"),
  正在读取选中文本: entry("正在讀取選中文字", "Reading Selected Text", "선택한 텍스트 읽는 중", "選択テキストを読み込み中", "Lecture du texte sélectionné", "Lese ausgewählten Text", "Leyendo texto seleccionado"),
  未选中: entry("未選中", "Not Selected", "선택 안 됨", "未選択", "Non sélectionné", "Nicht ausgewählt", "No seleccionado"),
  不支持: entry("不支援", "Not Supported", "지원 안 됨", "非対応", "Non pris en charge", "Nicht unterstützt", "No compatible"),
  失败: entry("失敗", "Failed", "실패", "失敗", "Échec", "Fehlgeschlagen", "Error"),
  已读取: entry("已讀取", "Read", "읽음", "読み取り済み", "Lu", "Gelesen", "Leído"),
  不能使用同一个快捷键: entry("不能使用同一個快捷鍵", "cannot use the same hotkey", "같은 단축키를 사용할 수 없습니다", "同じショートカットは使用できません", "ne peuvent pas utiliser le même raccourci", "können nicht dasselbe Tastenkürzel verwenden", "no pueden usar el mismo atajo"),
  划词和截图: entry("劃詞和截圖", "Selection and screenshot", "선택 및 스크린샷", "選択とスクリーンショット", "Sélection et capture", "Auswahl und Screenshot", "Selección y captura"),
  截图和输入翻译: entry("截圖和輸入翻譯", "Screenshot and input translation", "스크린샷 및 입력 번역", "スクリーンショットと入力翻訳", "Capture et traduction de saisie", "Screenshot und Eingabeübersetzung", "Captura y traducción de entrada"),
  输入翻译和划词: entry("輸入翻譯和劃詞", "Input translation and selection", "입력 번역 및 선택", "入力翻訳と選択", "Traduction de saisie et sélection", "Eingabeübersetzung und Auswahl", "Traducción de entrada y selección"),
  重置默认设置: entry("重設預設設定", "Reset Default Settings", "기본 설정 초기화", "既定設定にリセット", "Réinitialiser les paramètres", "Standardwerte zurücksetzen", "Restablecer configuración"),
  正在重置默认设置: entry("正在重設預設設定", "Resetting Default Settings", "기본 설정을 초기화하는 중", "既定設定をリセット中", "Réinitialisation des paramètres", "Setze Standardwerte zurück", "Restableciendo configuración"),
  重置默认设置失败: entry("重設預設設定失敗", "Failed to Reset Default Settings", "기본 설정 초기화 실패", "既定設定のリセットに失敗", "Échec de la réinitialisation", "Zurücksetzen fehlgeschlagen", "No se pudo restablecer"),
};

function languageLabelTranslation(value: string, locale: TranslationLanguage): string {
  return translations[value]?.[locale] || value;
}

function localizePatterns(text: string, locale: TranslationLanguage): string {
  let result = text;
  const sourceTarget = text.match(/^源语言：(.+?)\s*→\s*目标语言：(.+)$/);
  if (sourceTarget) {
    return `${translations["源语言"][locale]}${languageLabelTranslation(sourceTarget[1], locale)} → ${translations["目标语言"][locale]}${languageLabelTranslation(sourceTarget[2], locale)}`;
  }

  result = result.replace(
    /(\d+(?:\.\d+)?)\s*字符/g,
    (_match, count) =>
      `${count} ${
        {
          "zh-Hans": "字符",
          "zh-Hant": "字元",
          en: "characters",
          ko: "자",
          ja: "文字",
          fr: "caractères",
          de: "Zeichen",
          es: "caracteres",
        }[locale]
      }`,
  );

  const conflict = text.match(/^(.+?)不能使用同一个快捷键$/);
  if (conflict) {
    return `${languageLabelTranslation(conflict[1], locale)} ${translations["不能使用同一个快捷键"]?.[locale] || "cannot use the same hotkey"}`;
  }

  return result;
}

export function translateText(source: string, language: UiLanguage): string {
  const locale = resolveUiLanguage(language);
  if (locale === "zh-Hans") {
    return source;
  }
  const trimmed = source.trim();
  const exact = translations[trimmed]?.[locale];
  if (exact !== undefined) {
    const leading = source.slice(0, source.indexOf(trimmed));
    const trailing = source.slice(source.indexOf(trimmed) + trimmed.length);
    return `${leading}${exact}${trailing}`;
  }
  return localizePatterns(source, locale);
}

const translatedAttributes = [
  "aria-label",
  "aria-description",
  "title",
  "placeholder",
  "alt",
];

const originalText = new WeakMap<Text, string>();
const originalAttributes = new WeakMap<Element, Map<string, string>>();

function localizeTextNode(node: Text, language: UiLanguage) {
  const source = originalText.get(node) ?? node.nodeValue ?? "";
  originalText.set(node, source);
  const next = translateText(source, language);
  if (next !== source) {
    node.nodeValue = next;
  }
}

function localizeElement(element: Element, language: UiLanguage) {
  let attrs = originalAttributes.get(element);
  if (!attrs) {
    attrs = new Map();
    originalAttributes.set(element, attrs);
  }
  for (const name of translatedAttributes) {
    const current = element.getAttribute(name);
    if (current === null) {
      continue;
    }
    const source = attrs.get(name) ?? current;
    attrs.set(name, source);
    const next = translateText(source, language);
    if (next !== source) {
      element.setAttribute(name, next);
    }
  }
  if (element.closest("input, textarea, select, [contenteditable='true']")) {
    return;
  }
}

function localizeNode(node: Node, language: UiLanguage) {
  if (node.nodeType === Node.TEXT_NODE) {
    localizeTextNode(node as Text, language);
    return;
  }
  if (node.nodeType !== Node.ELEMENT_NODE) {
    return;
  }
  const element = node as Element;
  localizeElement(element, language);
  if (element.closest("input, textarea, select, [contenteditable='true']")) {
    return;
  }
  element.childNodes.forEach((child) => localizeNode(child, language));
}

let translationObserver: MutationObserver | null = null;

export function applyDocumentTranslation(language: UiLanguage) {
  const locale = resolveUiLanguage(language);
  document.documentElement.lang = locale;
  localizeNode(document.body, language);
  translationObserver?.disconnect();
  translationObserver = new MutationObserver((mutations) => {
    for (const mutation of mutations) {
      if (mutation.type === "attributes" && mutation.target instanceof Element) {
        localizeElement(mutation.target, language);
      }
      for (const node of mutation.addedNodes) {
        localizeNode(node, language);
      }
    }
  });
  translationObserver.observe(document.body, {
    childList: true,
    subtree: true,
    attributes: true,
    attributeFilter: translatedAttributes,
  });
}
