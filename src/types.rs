use strum::FromRepr;

pub enum RefType {
    SoftOwned,
    HardOwned,
    SoftPointer,
    HardPointer,
}

#[derive(FromRepr, Debug, PartialEq)]
#[repr(u16)]
pub enum CodePage {
    UTF8,
    USAscii,
    ISO8859_1,
    ISO8859_2,
    ISO8859_3,
    ISO8859_4,
    ISO8859_5,
    ISO8859_6,
    ISO8859_7,
    ISO8859_8,
    ISO8859_9,
    CP437, // Dos English,
    CP850, // Dos Latin 1,
    CP852, // Dos Central European,
    CP855, // Dos Cyrillic,
    CP857, // Dos Turkish,
    CP860, // Dos Portuguese,
    CP861, // Dos Icelandic,
    CP863, // Dos Hebrew,
    CP864, // Dos Arabic IBM,
    CP865, // Dos Nordic,
    CP869, // Dos Greek,
    CP932, // Dos Japanese,
    Macintosh,
    BIG5,
    CP949,
    JOHAB,
    CP866,    // Russian,
    ANSI1250, // Windows Central
    ANSI1251, // Windows Cyrillic
    ANSI1252, // Windows Western European
    GB2312,   // Windows EUC-CN Chinese
    ANSI1253, // Windows Greek
    ANSI1254, // Windows Turkish
    ANSI1255, // Windows Hebrew
    ANSI1256, // Windows Arabic
    ANSI1257, // Windows Baltic
    ANSI874,  // Windows Thai
    ANSI932,  // Windows Japanese
    ANSI936,  // Windows Simplified Chinese
    ANSI949,  // Windows Korean Wansung
    ANSI950,  // Windows Trad Chinese
    ANSI1361, // Windows Korean Wansung
    UTF16,    // Default Since R2007
    ANSI1258, // Windows Vietnamese
}
