/// Application category definitions with associated MIME types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppCategory {
    WebBrowser,
    EmailClient,
    FileManager,
    TerminalEmulator,
    TextEditor,
    MusicPlayer,
    VideoPlayer,
    ImageViewer,
    DocumentViewer,
    ArchiveManager,
    Calculator,
    Calendar,
    WordProcessor,
    Spreadsheet,
}

impl AppCategory {
    /// Returns all available categories in display order
    pub fn all() -> Vec<Self> {
        vec![
            Self::WebBrowser,
            Self::EmailClient,
            Self::FileManager,
            Self::TerminalEmulator,
            Self::TextEditor,
            Self::MusicPlayer,
            Self::VideoPlayer,
            Self::ImageViewer,
            Self::DocumentViewer,
            Self::ArchiveManager,
            Self::Calculator,
            Self::Calendar,
            Self::WordProcessor,
            Self::Spreadsheet,
        ]
    }

    /// Human-readable name for the category
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::WebBrowser => "Web Browser",
            Self::EmailClient => "Email Client",
            Self::FileManager => "File Manager",
            Self::TerminalEmulator => "Terminal",
            Self::TextEditor => "Text Editor",
            Self::MusicPlayer => "Music Player",
            Self::VideoPlayer => "Video Player",
            Self::ImageViewer => "Image Viewer",
            Self::DocumentViewer => "Document Viewer",
            Self::ArchiveManager => "Archive Manager",
            Self::Calculator => "Calculator",
            Self::Calendar => "Calendar",
            Self::WordProcessor => "Word Processor",
            Self::Spreadsheet => "Spreadsheet",
        }
    }

    /// Icon name for this category (freedesktop icon spec)
    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::WebBrowser => "web-browser",
            Self::EmailClient => "mail-client",
            Self::FileManager => "system-file-manager",
            Self::TerminalEmulator => "utilities-terminal",
            Self::TextEditor => "accessories-text-editor",
            Self::MusicPlayer => "applications-multimedia",
            Self::VideoPlayer => "video-x-generic",
            Self::ImageViewer => "image-x-generic",
            Self::DocumentViewer => "x-office-document",
            Self::ArchiveManager => "package-x-generic",
            Self::Calculator => "accessories-calculator",
            Self::Calendar => "x-office-calendar",
            Self::WordProcessor => "x-office-document",
            Self::Spreadsheet => "x-office-spreadsheet",
        }
    }

    /// Primary MIME types for this category (used for setting defaults)
    pub fn primary_mime_types(&self) -> Vec<&'static str> {
        match self {
            Self::WebBrowser => vec![
                "x-scheme-handler/http",
                "x-scheme-handler/https",
                "text/html",
                "application/xhtml+xml",
            ],
            Self::EmailClient => vec![
                "x-scheme-handler/mailto",
                "application/x-extension-eml",
                "message/rfc822",
            ],
            Self::FileManager => vec!["inode/directory"],
            Self::TerminalEmulator => vec![], // Uses Categories instead
            Self::TextEditor => vec!["text/plain"],
            Self::MusicPlayer => vec![
                "audio/mpeg",
                "audio/mp3",
                "audio/ogg",
                "audio/flac",
                "audio/x-wav",
                "audio/x-vorbis+ogg",
            ],
            Self::VideoPlayer => vec![
                "video/mp4",
                "video/x-matroska",
                "video/webm",
                "video/mpeg",
                "video/x-msvideo",
                "video/quicktime",
            ],
            Self::ImageViewer => vec![
                "image/png",
                "image/jpeg",
                "image/gif",
                "image/webp",
                "image/bmp",
                "image/svg+xml",
            ],
            Self::DocumentViewer => vec![
                "application/pdf",
                "application/postscript",
                "application/x-bzpdf",
                "application/x-gzpdf",
            ],
            Self::ArchiveManager => vec![
                "application/zip",
                "application/x-tar",
                "application/gzip",
                "application/x-gzip",
                "application/x-bzip2",
                "application/x-xz",
                "application/x-7z-compressed",
                "application/vnd.rar",
                "application/x-rar",
            ],
            Self::Calculator => vec![], // Uses Categories instead
            Self::Calendar => vec![
                "text/calendar",
                "application/x-extension-ics",
            ],
            Self::WordProcessor => vec![
                "application/msword",
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                "application/vnd.oasis.opendocument.text",
                "application/rtf",
            ],
            Self::Spreadsheet => vec![
                "application/vnd.ms-excel",
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                "application/vnd.oasis.opendocument.spreadsheet",
                "text/csv",
            ],
        }
    }

    /// Extended MIME types (shown in drill-down view)
    pub fn extended_mime_types(&self) -> Vec<&'static str> {
        match self {
            Self::MusicPlayer => vec![
                "audio/aac",
                "audio/mp4",
                "audio/x-m4a",
                "audio/x-aiff",
                "audio/x-ape",
                "audio/opus",
                "audio/x-opus+ogg",
            ],
            Self::VideoPlayer => vec![
                "video/x-flv",
                "video/3gpp",
                "video/ogg",
                "video/x-ogm+ogg",
            ],
            Self::ImageViewer => vec![
                "image/tiff",
                "image/x-icon",
                "image/heic",
                "image/heif",
                "image/avif",
            ],
            Self::TextEditor => vec![
                "text/x-csrc",
                "text/x-c++src",
                "text/x-python",
                "text/x-shellscript",
                "text/x-rust",
                "application/json",
                "application/xml",
                "text/markdown",
                "text/x-ini",
                "application/x-wine-extension-ini",
                "text/x-yaml",
                "text/x-toml",
                "text/x-cmake",
                "text/x-makefile",
                "text/x-log",
                "application/x-desktop",
                "text/x-java",
                "text/x-javascript",
                "text/x-typescript",
                "text/css",
                "text/html",
            ],
            _ => vec![],
        }
    }

    /// Desktop categories to search for (for apps without MIME types)
    pub fn desktop_categories(&self) -> Vec<&'static str> {
        match self {
            Self::Calculator => vec!["Calculator"],
            Self::Calendar => vec!["Calendar"],
            Self::TerminalEmulator => vec!["TerminalEmulator"],
            Self::WebBrowser => vec!["WebBrowser"],
            Self::EmailClient => vec!["Email"],
            Self::FileManager => vec!["FileManager"],
            Self::TextEditor => vec!["TextEditor"],
            Self::MusicPlayer => vec!["Audio", "Music", "Player"],
            Self::VideoPlayer => vec!["Video", "AudioVideo"],
            Self::ImageViewer => vec!["Viewer", "Graphics"],
            Self::DocumentViewer => vec!["Viewer", "Office"],
            Self::ArchiveManager => vec!["Archiving", "Compression"],
            Self::WordProcessor => vec!["WordProcessor", "Office"],
            Self::Spreadsheet => vec!["Spreadsheet", "Office"],
        }
    }

    /// Get the primary MIME type used for querying the current default
    pub fn default_query_mime(&self) -> Option<&'static str> {
        self.primary_mime_types().first().copied()
    }
}
