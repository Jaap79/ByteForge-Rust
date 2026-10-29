; ByteForge 1.0 RC2 beta 9 - pure FASM / Win32 Unicode
; Build: fasm src\ByteForge.asm dist\Byteforge.exe
;
; Goals:
; - small single EXE, no CRT, native-first Win32 implementation
; - GUI text editor using native EDIT control
; - Unicode internally, UTF-8/UTF-16LE/UTF-16BE BOM detection
; - Open / Save / Save As / New
; - Ctrl+N, Ctrl+O, Ctrl+S, Ctrl+F
; - Drag & Drop file onto window
; - Explorer/Open With startup file association support via GetCommandLineW
;   and CommandLineToArgvW; startup, drag/drop and File->Open all share
;   ByteForge's OpenFileDocument implementation
; - Selectable checksum window with optional expected MD5/SHA-256
;   verification, one-shot "no more matches" notice, and editor Ctrl+/- zoom.
; - RC2 beta 6: closing, New, Open and Drag & Drop prompt to save unsaved
;   changes; new documents use Save As, existing documents save in place.
; - RC2 beta 8: empty new documents no longer trigger an unnecessary save
;   prompt, and the experimental preview/parser work was removed again.
; - RC2 beta 9: right-click editor menu with Cut/Copy/Paste/Find More and
;   selectable text highlighting from Edit and context menus.
; - Security issues and bugs can be reported to jaapengel79@proton.me.
; - Window title contains full path + '*' when modified
; - Status bar shows line/column left, version right
; - Stable dark-gray mode for editor + status bar
;
; Notes:
; - Native-first policy: use standard Windows DLLs and keep optional parser
;   experiments outside this mainline source.
; - Classic menu bar remains system-drawn for compactness/stability.
; - Executable version resource identifies ByteForge.

format PE GUI 4.0
entry start

include 'api\win32wx.inc'


ID_EDIT             = 100
IDI_APP             = 1
ID_STATUS           = 101
ID_STATUS_INFO      = 102
ID_STATUS_VERSION   = 103
ID_HEX              = 104
IDM_NEW             = 201
IDM_OPEN            = 202
IDM_SAVE            = 203
IDM_SAVEAS          = 204
IDM_FIND            = 205
IDM_DARK            = 206
IDM_EXIT            = 207
IDM_UNDO            = 208
IDM_REDO            = 209
IDM_CUT             = 210
IDM_COPY            = 211
IDM_PASTE           = 212
IDM_SELECTALL       = 213
IDM_REPLACE         = 214
IDM_WORDWRAP        = 215
IDM_LINENUMBERS     = 216
IDM_ABOUT           = 217
IDM_HEXVIEW         = 218
IDM_SHA256          = 219
IDM_JSONFMT         = 220
IDM_B64DEC          = 221
IDM_URLDEC          = 222
IDM_XOR             = 223
IDM_ROT13           = 224
IDM_HEX_LEFT        = 225
IDM_HEX_RIGHT       = 226
IDM_HEX_BELOW       = 227
IDM_HEX_CLOSE       = 228
IDM_FONT_DEFAULT    = 229
IDM_FONT_SEGOE      = 230
IDM_FONT_GEORGIA    = 231
IDM_FONT_CONSOLAS   = 232
IDM_ZOOM_IN         = 233
IDM_ZOOM_OUT        = 234
IDM_ZOOM_RESET      = 235
IDM_JUMP_LINECHAR   = 236
IDM_FILE_INFO       = 237
IDM_CHECK_UPDATES   = 238
IDM_FIND_MORE       = 239
IDM_HILITE_RED      = 240
IDM_HILITE_ORANGE   = 241
IDM_HILITE_YELLOW   = 242
IDM_HILITE_GREEN    = 243
IDM_HILITE_CLEAR    = 244
ID_BTN_CHECKSUM_COMPARE = 301
ID_BTN_CHECKSUM_CLOSE   = 302
ID_ED_MD5_ACTUAL        = 303
ID_ED_MD5_EXPECTED      = 304
ID_ED_SHA_ACTUAL        = 305
ID_ED_SHA_EXPECTED      = 306
ID_ED_CHECKSUM_RESULT   = 307
ID_BTN_JUMP_GO          = 321
ID_BTN_JUMP_CANCEL      = 322
ID_ED_JUMP_LINE         = 323
ID_ED_JUMP_CHAR         = 324
ID_BTN_FILEINFO_CLOSE   = 331
ID_ED_FILEINFO_TEXT     = 332
ID_FIND_DIALOG_TEXT     = 480
ACCEL_COUNT         = 40
VK_OEM_PLUS         = 0BBh
VK_OEM_MINUS        = 0BDh
VK_ADD              = 06Bh
VK_SUBTRACT         = 06Dh
TIMER_STATUS_ID     = 1
TIMER_STATUS_MS     = 100
STATUS_H            = 22
EM_SETBKGNDCOLOR    = 1091
EM_FINDTEXTEXW       = 1148          ; RichEdit EM_FINDTEXTEXW = WM_USER + 124
EM_EXLIMITTEXT       = 1077          ; RichEdit EM_EXLIMITTEXT = WM_USER + 53
EM_HIDESELECTION     = 1087          ; RichEdit EM_HIDESELECTION = WM_USER + 63
EM_GETTEXTEX         = 1118          ; RichEdit EM_GETTEXTEX = WM_USER + 94
ENC_UTF8            = 1
ENC_UTF16LE         = 2
ENC_UTF16BE         = 3
EOL_UNKNOWN         = 0
EOL_CRLF            = 1
EOL_LF              = 2
EOL_CR              = 3
CLEARTYPE_QUALITY   = 5
INVALID_FILE_ATTRIBUTES = 0FFFFFFFFh
WM_SETREDRAW        = 11
EM_SETEVENTMASK     = 1093
ENM_CHANGE          = 1
EM_SETCHARFORMAT    = 1092
EM_GETTEXTRANGE     = 1099
WM_CONTEXTMENU      = 007Bh
WM_INITMENUPOPUP    = 0117h
TPM_RETURNCMD       = 0100h
TPM_RIGHTBUTTON     = 0002h
SCF_DEFAULT         = 0
SCF_SELECTION       = 1
SCF_ALL             = 4
CFM_COLOR           = 40000000h
CFM_FACE            = 20000000h
CFM_BOLD            = 00000001h
CFM_ITALIC          = 00000002h
CFM_SIZE            = 80000000h
CFM_BACKCOLOR       = 04000000h
CFE_AUTOBACKCOLOR   = 04000000h
CFE_BOLD            = 00000001h
CFE_ITALIC          = 00000002h
CHARFORMATW_SIZE    = 92
CHARFORMAT2W_SIZE   = 116
HEX_OFF             = 0
HEX_LEFT            = 1
HEX_RIGHT           = 2
HEX_BELOW           = 3
HEX_PREVIEW_CHARS   = 32768
HEX_PREVIEW_BYTES   = 65536
ES_READONLY          = 0800h
HEX_OUT_BYTES       = 786432
PROV_RSA_AES        = 24
CRYPT_VERIFYCONTEXT = 0F0000000h
CALG_MD5            = 00008003h
CALG_SHA_256        = 0000800Ch
HP_HASHVAL          = 2
HASH_CHUNK          = 32768
UPDATE_BUF_SIZE     = 4096
INTERNET_OPEN_TYPE_PRECONFIG = 0
INTERNET_FLAG_RELOAD = 80000000h
INTERNET_FLAG_NO_CACHE_WRITE = 04000000h
FONT_DEFAULT        = 0
FONT_SEGOE          = 1
FONT_GEORGIA        = 2
FONT_CONSOLAS       = 3
LOCAL_VER_MAJOR     = 1
LOCAL_VER_MINOR     = 0
LOCAL_VER_PATCH     = 0
LOCAL_VER_BUILD     = 10

section '.data' data readable writeable

VERSION_W           du 'ByteForge 1.0 RC2 beta 9',0
APP_CLASS           du 'ByteForge10RC2B9Class',0
APP_TITLE           du 'ByteForge 1.0 RC2 beta 9',0
CHECKSUM_CLASS      du 'ByteForge10RC2B9ChecksumClass',0
JUMP_CLASS          du 'ByteForge10RC2B9JumpClass',0
FILEINFO_CLASS      du 'ByteForge10RC2B9FileInfoClass',0
EDIT_CLASS          du 'RICHEDIT50W',0
WINEDIT_CLASS       du 'EDIT',0
BUTTON_CLASS        du 'BUTTON',0
MSFTEDIT_DLL        du 'Msftedit.dll',0
STATIC_CLASS        du 'STATIC',0
UNTITLED_W          du 'Untitled',0
STAR_SPACE_W        du '* ',0
TITLE_SEP_W         du ' - ',0
DARK_ON_W           du 'Dark mode: on',0
DARK_OFF_W          du 'Dark mode: off',0
ENC_UTF8_W          du 'UTF-8',0
ENC_UTF16LE_W       du 'UTF-16LE',0
ENC_UTF16BE_W       du 'UTF-16BE',0
EOL_UNKNOWN_W       du 'EOL ?',0
EOL_CRLF_W          du 'CRLF',0
EOL_LF_W            du 'LF',0
EOL_CR_W            du 'CR',0
findMsgString       du 'commdlg_FindReplace',0
HEX_FONT_FACE       du 'Courier New',0
FONT_DEFAULT_FACE   du 'Courier New',0
FONT_SEGOE_FACE     du 'Segoe UI',0
FONT_GEORGIA_FACE   du 'Georgia',0
FONT_CONSOLAS_FACE  du 'Consolas',0

hInst       dd 0
hAccel      dd 0
hwndMain    dd 0
hwndEdit    dd 0
parentHwnd  dd 0
editStyle   dd 0
hwndStatus  dd 0
hwndStatusInfo dd 0
hwndStatusVer dd 0
hwndHex     dd 0
hHexFont    dd 0
hEditorFont dd 0
hMenuMain   dd 0
hMenuFile   dd 0
hMenuView   dd 0
hMenuFont   dd 0
hMenuEdit   dd 0
hMenuTools  dd 0
hMenuHex    dd 0
hMenuHelp   dd 0
hMenuHighlight dd 0
hFindDlg    dd 0
hDarkBrush  dd 0
hStatBrush  dd 0
hFont       dd 0
hRichEdit   dd 0
loading     dd 0
modified    dd 0
darkmode    dd 1
wordwrap    dd 0
editorFontChoice dd FONT_DEFAULT
linenums    dd 0
hexView     dd HEX_OFF
hexDirty    dd 0
hexSplitW   dd 360
hexSplitH   dd 220
splitDrag   dd 0
mouseParam  dd 0
layoutRedraw dd TRUE
encoding    dd ENC_UTF8
lineEnding  dd EOL_CRLF
findMsg     dd 0
cmdArgc     dd 0
cmdArgv     dd 0
findNoMoreShown dd 0
editorZoom  dd 0
hwndChecksum dd 0
hwndMd5Actual dd 0
hwndMd5Expected dd 0
hwndShaActual dd 0
hwndShaExpected dd 0
hwndChecksumResult dd 0
hwndJump    dd 0
hwndJumpLine dd 0
hwndJumpChar dd 0
jumpCharIdx dd 0
hwndFileInfo dd 0
hwndFileInfoText dd 0
fileInfoStatusStart dd 0
fileInfoStatusEnd dd 0
fileInfoStatusColor dd 0

wc          WNDCLASS
msg         MSG
getTextExCb dd 0
getTextExFlags dd 0
getTextExCodepage dd 1200
getTextExDefaultChar dd 0
getTextExUsedDefChar dd 0
ofn         OPENFILENAME
fr          FINDREPLACE
ft_chrg_min dd 0
ft_chrg_max dd 0
ft_text     dd 0
ft_res_min  dd 0
ft_res_max  dd 0
tr_chrg_min dd 0
tr_chrg_max dd 0
tr_text     dd 0
hx_chrg_min dd 0
hx_chrg_max dd 0
hx_text     dd 0
hx_res_min  dd 0
hx_res_max  dd 0
fileBomBytes dd 0
hexSelStart dd 0
hexSelEnd   dd 0
hexBaseLen  dd 0
hexLineNo   dd 0
hexColNo    dd 0
findBuf     rw 256
lastFindBuf rw 256
jumpLineBuf rw 32
jumpCharBuf rw 32
hexOffsetNeedle rw 16
filePath    rw 260
titleBuf    rw 640
statusBuf   rw 128
numBuf      rw 16
versionBuf  rw 32
lineIdx     dd 0
selStart    dd 0
selEnd      dd 0
bytesIO     dd 0
fileSize    dd 0
tmpPtr      dd 0
tmpLen      dd 0
tmpSize     dd 0
tmpAttr     dd 0
tmpOutLen   dd 0
hFileRead   dd 0
hFileWrite  dd 0
rawPtr      dd 0
textPtr     dd 0
convPtr     dd 0
clientW     dd 0
clientH     dd 0
paneX       dd 0
paneY       dd 0
paneW       dd 0
paneH       dd 0
leftW       dd 185
rightW      dd 135
midX        dd 170
midW        dd 0
encLabel    dd ENC_UTF8_W
eolLabel    dd EOL_CRLF_W
scanPtr     dd 0
hexTextPtr  dd 0
hexBytePtr  dd 0
hexOutPtr   dd 0
hexChars    dd 0
hexBytes    dd 0
hexLimitMsg dd 0
hexLineBytes dd 0
hexBytesPerLine dd 16
hexAsciiCol dd 62
hexLineChars dd 80
hexSourceBuf dd 0       ; 0 = file on disk, 1 = current editor buffer
hProv       dd 0
hHashMd5    dd 0
hHashSha    dd 0
hHashFile   dd 0
hashRead    dd 0
hashLen     dd 0
hInternet   dd 0
hUpdateUrl  dd 0
updateRead  dd 0
remoteVerMajor dd 0
remoteVerMinor dd 0
remoteVerPatch dd 0
remoteVerBuild dd 0
hashBuf     rb HASH_CHUNK
updateBuf   rb UPDATE_BUF_SIZE
md5Bin      rb 16
shaBin      rb 32
md5Hex      rw 33
shaHex      rw 65
remoteVersionText rw 32
expectedMd5 rw 80
expectedSha rw 96
checksumResultBuf rw 384
updateMsgBuf rw 1024
fileInfoBuf rw 4096
fileInfoAttr rb 36
localFileTime rb 8
fileSysTime rb 16
textColor   dd 0
highlightColor dd 0
ctxX        dd 0
ctxY        dd 0
pt_x        dd 0
pt_y        dd 0
charFmt     rb CHARFORMAT2W_SIZE

menuFileTxt du '&File',0
menuViewTxt du '&View',0
menuEditTxt du '&Edit',0
menuToolsTxt du '&Tools',0
menuHelpTxt du '&Help',0
menuNewTxt  du '&New',9,'Ctrl+N',0
menuOpenTxt du '&Open...',9,'Ctrl+O',0
menuSaveTxt du '&Save',9,'Ctrl+S',0
menuSaveAsTxt du 'Save &As...',9,'Ctrl+Shift+S',0
menuFileInfoTxt du 'File &Info...',9,'Ctrl+I',0
menuFindTxt du '&Find...',9,'Ctrl+F',0
menuFindMoreTxt du 'Find &More',9,'F3',0
menuReplaceTxt du '&Replace...',9,'Ctrl+H',0
menuJumpTxt du '&Jump to line/char...',9,'Ctrl+G',0
menuHighlightTxt du '&Highlight',0
menuHighlightClearTxt du '&Remove highlight',9,'Ctrl+0',0
menuHighlightRedTxt du '&Red',9,'Ctrl+1',0
menuHighlightOrangeTxt du '&Orange',9,'Ctrl+2',0
menuHighlightYellowTxt du '&Yellow',9,'Ctrl+3',0
menuHighlightGreenTxt du '&Green',9,'Ctrl+4',0
menuUndoTxt du '&Undo',9,'Ctrl+Z',0
menuRedoTxt du '&Redo',9,'Ctrl+Y',0
menuCutTxt du 'Cu&t',9,'Ctrl+X',0
menuCopyTxt du '&Copy',9,'Ctrl+C',0
menuPasteTxt du '&Paste',9,'Ctrl+V',0
menuSelectAllTxt du 'Select &All',9,'Ctrl+A',0
menuDarkTxt du '&Dark gray mode',9,'Ctrl+D',0
menuWrapTxt du '&Word wrap',9,'Ctrl+W',0
menuZoomInTxt du 'Zoom &In',9,'Ctrl++',0
menuZoomOutTxt du 'Zoom &Out',9,'Ctrl+-',0
menuZoomResetTxt du 'Zoom &Reset',9,'Ctrl+Shift+0',0
menuFontTxt du '&Font',0
menuFontDefaultTxt du '&Default',9,'Ctrl+Shift+1',0
menuFontSegoeTxt du '&Segoe UI',9,'Ctrl+Shift+2',0
menuFontGeorgiaTxt du '&Georgia',9,'Ctrl+Shift+3',0
menuFontConsolasTxt du '&Consolas',9,'Ctrl+Shift+4',0
menuExitTxt du 'E&xit',9,'Alt+F4',0
menuAboutTxt du '&About ByteForge',9,'F1',0
menuCheckUpdatesTxt du '&Check for Updates...',9,'Ctrl+U',0
menuHexTxt du '&Hex Viewer',0
menuHexLeftTxt du 'Hex view &left',9,'Ctrl+Shift+L',0
menuHexRightTxt du 'Hex view &right',9,'Ctrl+Shift+R',0
menuHexBelowTxt du 'Hex view &below',9,'Ctrl+Shift+B',0
menuHexCloseTxt du '&Close hex view',9,'Ctrl+Shift+X',0
menuChecksumTxt du '&Checksum of File',9,'Ctrl+K',0
aboutTxt du 'ByteForge 1.0 RC2 beta 9',13,10,'Small and fast text editor without fluff.',13,10,'Single EXE, no CRT, native-first Win32 code.',13,10,13,10,'Security issues and bugs: jaapengel79@proton.me',0
savePromptTxt du 'Save changes before continuing?',0
findNotFoundTxt du 'No more matches found.',0
SAVE_FAIL_PREFIX du 'Save failed. GetLastError = ',0
HEX_LIMIT_TXT du 'Hex preview limited to first 64 KB.',0
MD5_TXT du 'MD5 ',0
SHA256_TXT du ' SHA256 ',0
HASH_NONE_TXT du 'MD5 - SHA256 -',0
checksumTitleTxt du 'Checksum',0
checksumNoFileTxt du 'No file is currently open.',0
checksumNoHashTxt du 'No checksum available.',0
checksumMd5Lbl du 'MD5:',13,10,0
checksumShaLbl du 13,10,13,10,'SHA-256:',13,10,0
checksumFileNoteTxt du 13,10,13,10,'Source: file on disk.',0
checksumBufferNoteTxt du 13,10,13,10,'Source: current editor buffer (unsaved contents).',0
checksumMd5LabelTxt du 'Actual MD5',0
checksumShaLabelTxt du 'Actual SHA-256',0
checksumExpectedMd5Txt du 'Expected MD5 (optional)',0
checksumExpectedShaTxt du 'Expected SHA-256 (optional)',0
checksumCompareTxt du '&Compare',0
checksumCloseTxt du '&Close',0
checksumResultReadyTxt du 'Enter an expected MD5 and/or SHA-256, then click Compare.',0
checksumMd5MatchTxt du 'MD5: MATCH',13,10,0
checksumMd5MismatchTxt du 'MD5: MISMATCH',13,10,0
checksumMd5SkippedTxt du 'MD5: not checked',13,10,0
checksumShaMatchTxt du 'SHA-256: MATCH',13,10,0
checksumShaMismatchTxt du 'SHA-256: MISMATCH',13,10,0
checksumShaSkippedTxt du 'SHA-256: not checked',13,10,0
updateTitleTxt du 'Check for Updates',0
updateAgentTxt du 'ByteForge update check',0
updateUrlTxt du 'https://raw.githubusercontent.com/Jaap79/ByteForge/main/version.json',0
updateCurrentTxt du 'ByteForge 1.0 RC2 beta 9 (1.0.0.10)',0
updateAvailableTxt du 'A newer ByteForge version is available.',13,10,13,10,'Current: ',0
updateCurrentLatestTxt du 'ByteForge is up to date.',13,10,13,10,'Current: ',0
updateLatestTxt du 13,10,'Latest: ',0
updateDownloadTxt du 13,10,13,10,'Download: https://github.com/Jaap79/ByteForge/releases/latest',0
updateFailTxt du 'Could not check for updates.',13,10,13,10,'Please check:',13,10,'https://github.com/Jaap79/ByteForge/releases/latest',0
updateVersionKey db '"version"',0
jumpTitleTxt du 'Jump to line/char',0
jumpLineTxt du 'Line',0
jumpCharTxt du 'Character (optional)',0
jumpGoTxt du '&Jump',0
jumpCancelTxt du '&Cancel',0
fileInfoTitleTxt du 'File Info',0
fileInfoCloseTxt du '&Close',0
fileInfoPathTxt du 'Path: ',0
fileInfoFormatTxt du 13,10,'Format: ',0
fileInfoFormatSepTxt du ' / ',0
fileInfoBomTxt du ', BOM bytes: ',0
fileInfoLinesTxt du 13,10,'Total lines: ',0
fileInfoCharsTxt du 13,10,'Total characters: ',0
fileInfoSizeTxt du 13,10,'File size: ',0
fileInfoBytesTxt du ' bytes',0
fileInfoModifiedTxt du 13,10,'Modified in editor: ',0
fileInfoYesTxt du 'Yes',0
fileInfoNoTxt du 'No',0
fileInfoSavedTxt du 13,10,'Last saved: ',0
fileInfoNeverSavedTxt du 'Not saved yet',0
fileInfoUnsavedTxt du ' [Unsaved]',0
fileInfoUnavailableTxt du 'Unavailable',0
fileInfoChecksumSourceTxt du 13,10,13,10,'Checksum source: ',0
fileInfoMd5Txt du 13,10,'MD5: ',0
fileInfoShaTxt du 13,10,'SHA-256: ',0
fileInfoNoteTxt du 13,10,13,10,'Tip: these checksum fields are selectable/copyable.',0
fileInfoDashTxt du '-',0
HEX_SOURCE_FILE_W du 'Source: file on disk',13,10,0
HEX_SOURCE_BUFFER_W du 'Source: current editor buffer',13,10,0

filterTxt   du 'Text files (*.txt)',0,'*.txt',0,'All files (*.*)',0,'*.*',0,0
lnTxt       du 'Ln ',0
colTxt      du ', Col ',0
infoSepTxt  du '   ',0
hashSepTxt  du ' | ',0
HEX_DIGITS_W du '0123456789ABCDEF',0
HEX_HEADER_W du 'Offset    | Hex bytes                                        | ASCII',13,10,0

accels:
    db FVIRTKEY or FCONTROL,0
    dw 'N', IDM_NEW
    db FVIRTKEY or FCONTROL,0
    dw 'O', IDM_OPEN
    db FVIRTKEY or FCONTROL,0
    dw 'S', IDM_SAVE
    db FVIRTKEY or FCONTROL or FSHIFT,0
    dw 'S', IDM_SAVEAS
    db FVIRTKEY or FCONTROL,0
    dw 'I', IDM_FILE_INFO
    db FVIRTKEY or FCONTROL,0
    dw 'F', IDM_FIND
    db FVIRTKEY or FCONTROL,0
    dw 'H', IDM_REPLACE
    db FVIRTKEY,0
    dw VK_F3, IDM_FIND_MORE
    db FVIRTKEY or FCONTROL,0
    dw 'G', IDM_JUMP_LINECHAR
    db FVIRTKEY or FCONTROL,0
    dw 'A', IDM_SELECTALL
    db FVIRTKEY or FCONTROL,0
    dw 'Z', IDM_UNDO
    db FVIRTKEY or FCONTROL,0
    dw 'Y', IDM_REDO
    db FVIRTKEY or FCONTROL,0
    dw 'X', IDM_CUT
    db FVIRTKEY or FCONTROL,0
    dw 'C', IDM_COPY
    db FVIRTKEY or FCONTROL,0
    dw 'V', IDM_PASTE
    db FVIRTKEY or FCONTROL,0
    dw 'W', IDM_WORDWRAP
    db FVIRTKEY or FCONTROL,0
    dw 'D', IDM_DARK
    db FVIRTKEY or FCONTROL,0
    dw '0', IDM_HILITE_CLEAR
    db FVIRTKEY or FCONTROL,0
    dw '1', IDM_HILITE_RED
    db FVIRTKEY or FCONTROL,0
    dw '2', IDM_HILITE_ORANGE
    db FVIRTKEY or FCONTROL,0
    dw '3', IDM_HILITE_YELLOW
    db FVIRTKEY or FCONTROL,0
    dw '4', IDM_HILITE_GREEN
    db FVIRTKEY or FCONTROL,0
    dw VK_OEM_PLUS, IDM_ZOOM_IN
    db FVIRTKEY or FCONTROL or FSHIFT,0
    dw VK_OEM_PLUS, IDM_ZOOM_IN
    db FVIRTKEY or FCONTROL,0
    dw VK_OEM_MINUS, IDM_ZOOM_OUT
    db FVIRTKEY or FCONTROL,0
    dw VK_ADD, IDM_ZOOM_IN
    db FVIRTKEY or FCONTROL,0
    dw VK_SUBTRACT, IDM_ZOOM_OUT
    db FVIRTKEY or FCONTROL or FSHIFT,0
    dw '0', IDM_ZOOM_RESET
    db FVIRTKEY or FCONTROL or FSHIFT,0
    dw '1', IDM_FONT_DEFAULT
    db FVIRTKEY or FCONTROL or FSHIFT,0
    dw '2', IDM_FONT_SEGOE
    db FVIRTKEY or FCONTROL or FSHIFT,0
    dw '3', IDM_FONT_GEORGIA
    db FVIRTKEY or FCONTROL or FSHIFT,0
    dw '4', IDM_FONT_CONSOLAS
    db FVIRTKEY or FCONTROL or FSHIFT,0
    dw 'L', IDM_HEX_LEFT
    db FVIRTKEY or FCONTROL or FSHIFT,0
    dw 'R', IDM_HEX_RIGHT
    db FVIRTKEY or FCONTROL or FSHIFT,0
    dw 'B', IDM_HEX_BELOW
    db FVIRTKEY or FCONTROL or FSHIFT,0
    dw 'X', IDM_HEX_CLOSE
    db FVIRTKEY or FCONTROL,0
    dw 'K', IDM_SHA256
    db FVIRTKEY or FCONTROL,0
    dw 'U', IDM_CHECK_UPDATES
    db FVIRTKEY,0
    dw VK_F1, IDM_ABOUT
    db FVIRTKEY or FALT,0
    dw VK_F4, IDM_EXIT

section '.code' code readable executable

start:
    invoke GetModuleHandle,0
    mov [hInst],eax

    invoke LoadLibrary,MSFTEDIT_DLL
    mov [hRichEdit],eax

    invoke RegisterWindowMessage, findMsgString
    mov [findMsg],eax

    invoke CreateSolidBrush, 202020h
    mov [hDarkBrush],eax
    invoke CreateSolidBrush, 303030h
    mov [hStatBrush],eax

    invoke CreateFont, -16,0,0,0,FW_NORMAL,0,0,0,DEFAULT_CHARSET,OUT_DEFAULT_PRECIS,CLIP_DEFAULT_PRECIS,CLEARTYPE_QUALITY,FIXED_PITCH or FF_MODERN,0
    mov [hFont],eax
    call CreateEditorFont
    invoke CreateFont, -15,0,0,0,FW_NORMAL,0,0,0,DEFAULT_CHARSET,OUT_DEFAULT_PRECIS,CLIP_DEFAULT_PRECIS,CLEARTYPE_QUALITY,FIXED_PITCH or FF_MODERN,HEX_FONT_FACE
    mov [hHexFont],eax

    invoke LoadCursor,0,IDC_ARROW
    mov [wc.hCursor],eax
    mov [wc.style],CS_HREDRAW or CS_VREDRAW
    mov [wc.lpfnWndProc],WndProc
    mov [wc.cbClsExtra],0
    mov [wc.cbWndExtra],0
    mov eax,[hInst]
    mov [wc.hInstance],eax
    invoke LoadIcon,[hInst],IDI_APP
    mov [wc.hIcon],eax
    mov [wc.hbrBackground],COLOR_WINDOW+1
    mov [wc.lpszMenuName],0
    mov [wc.lpszClassName],APP_CLASS
    invoke RegisterClass,wc
    call RegisterChecksumClass
    call RegisterJumpClass
    call RegisterFileInfoClass

    call MakeMenu
    invoke CreateAcceleratorTable, accels, ACCEL_COUNT
    mov [hAccel], eax

    invoke CreateWindowEx,0,APP_CLASS,APP_TITLE,WS_OVERLAPPEDWINDOW or WS_VISIBLE or WS_CLIPCHILDREN,CW_USEDEFAULT,CW_USEDEFAULT,820,560,0,[hMenuMain],[hInst],0
    mov [hwndMain],eax
    call OpenCommandLineFile

.msg_loop:
    invoke GetMessage,msg,0,0,0
    test eax,eax
    jz .program_end

    cmp [hFindDlg],0
    je .no_find
    invoke IsDialogMessage,[hFindDlg],msg
    test eax,eax
    jnz .msg_loop
.no_find:
    cmp [hwndChecksum],0
    je .no_checksum
    invoke IsDialogMessage,[hwndChecksum],msg
    test eax,eax
    jnz .msg_loop
.no_checksum:
    cmp [hwndJump],0
    je .no_jump
    invoke IsDialogMessage,[hwndJump],msg
    test eax,eax
    jnz .msg_loop
.no_jump:
    cmp [hwndFileInfo],0
    je .no_fileinfo
    invoke IsDialogMessage,[hwndFileInfo],msg
    test eax,eax
    jnz .msg_loop
.no_fileinfo:
    cmp [hwndChecksum],0
    je .check_jump_focus
    invoke GetFocus
    cmp eax,[hwndChecksum]
    je .no_accel
    invoke IsChild,[hwndChecksum],eax
    test eax,eax
    jnz .no_accel
.check_jump_focus:
    cmp [hwndJump],0
    je .try_accel
    invoke GetFocus
    cmp eax,[hwndJump]
    je .no_accel
    invoke IsChild,[hwndJump],eax
    test eax,eax
    jnz .no_accel
.try_accel:
    cmp [hwndFileInfo],0
    je .try_accel_real
    invoke GetFocus
    cmp eax,[hwndFileInfo]
    je .no_accel
    invoke IsChild,[hwndFileInfo],eax
    test eax,eax
    jnz .no_accel
.try_accel_real:
    invoke TranslateAccelerator,[hwndMain],[hAccel],msg
    test eax,eax
    jnz .msg_loop

.no_accel:
    invoke TranslateMessage,msg
    invoke DispatchMessage,msg
    jmp .msg_loop

.program_end:
    cmp [hAccel],0
    je .no_accel_destroy
    invoke DestroyAcceleratorTable,[hAccel]
.no_accel_destroy:
    invoke DeleteObject,[hDarkBrush]
    invoke DeleteObject,[hStatBrush]
    invoke DeleteObject,[hFont]
    cmp [hEditorFont],0
    je .no_editor_font_delete
    invoke DeleteObject,[hEditorFont]
.no_editor_font_delete:
    cmp [hHexFont],0
    je .no_hex_font_delete
    invoke DeleteObject,[hHexFont]
.no_hex_font_delete:
    cmp [hRichEdit],0
    je .no_richedit_free
    invoke FreeLibrary,[hRichEdit]
.no_richedit_free:
    invoke ExitProcess,[msg.wParam]

proc WndProc uses ebx esi edi, hwnd,wmsg,wparam,lparam
    mov eax,[wmsg]
    cmp eax,[findMsg]
    je .findmsg
    cmp eax,WM_CREATE
    je .create
    cmp eax,WM_SIZE
    je .size
    cmp eax,WM_TIMER
    je .timer
    cmp eax,WM_KEYUP
    je .status_refresh
    cmp eax,WM_LBUTTONDOWN
    je .mouse_down
    cmp eax,WM_MOUSEMOVE
    je .mouse_move
    cmp eax,WM_LBUTTONUP
    je .mouse_up
    cmp eax,WM_CONTEXTMENU
    je .context_menu
    cmp eax,WM_INITMENUPOPUP
    je .init_menu_popup
    cmp eax,WM_COMMAND
    je .command
    cmp eax,WM_DROPFILES
    je .drop
    cmp eax,WM_CTLCOLOREDIT
    je .ctlcolor
    cmp eax,WM_CTLCOLORSTATIC
    je .ctlcolor_static
    cmp eax,WM_SETFOCUS
    je .focus
    cmp eax,WM_CLOSE
    je .close
    cmp eax,WM_DESTROY
    je .destroy
    invoke DefWindowProc,[hwnd],[wmsg],[wparam],[lparam]
    ret

.create:
    mov eax,[hwnd]
    mov [parentHwnd],eax
    call CreateEditorControl

    invoke CreateWindowEx,0,STATIC_CLASS,0,WS_CHILD or WS_VISIBLE or SS_LEFT,0,0,0,0,[hwnd],ID_STATUS,[hInst],0
    mov [hwndStatus],eax
    invoke SendMessage,[hwndStatus],WM_SETFONT,[hFont],TRUE
    invoke CreateWindowEx,0,STATIC_CLASS,0,WS_CHILD or WS_VISIBLE or SS_CENTER,0,0,0,0,[hwnd],ID_STATUS_INFO,[hInst],0
    mov [hwndStatusInfo],eax
    invoke SendMessage,[hwndStatusInfo],WM_SETFONT,[hFont],TRUE
    invoke CreateWindowEx,0,STATIC_CLASS,0,WS_CHILD or WS_VISIBLE or SS_RIGHT,0,0,0,0,[hwnd],ID_STATUS_VERSION,[hInst],0
    mov [hwndStatusVer],eax
    invoke SendMessage,[hwndStatusVer],WM_SETFONT,[hFont],TRUE

    invoke DragAcceptFiles,[hwnd],TRUE
    call ApplyTheme
    call UpdateTitle
    call UpdateStatus
    invoke SetTimer,[hwnd],TIMER_STATUS_ID,TIMER_STATUS_MS,0
    xor eax,eax
    ret

.timer:
    cmp [wparam],TIMER_STATUS_ID
    jne .default
.status_refresh:
    call UpdateStatus
    cmp [splitDrag],0
    jne .no_hex_timer
    cmp [hexDirty],0
    je .no_hex_timer
    call UpdateHexPreview
.no_hex_timer:
    xor eax,eax
    ret

.size:
    mov eax,[lparam]
    and eax,0FFFFh
    mov [clientW],eax
    mov eax,[lparam]
    shr eax,16
    sub eax,STATUS_H
    mov [clientH],eax
    call LayoutControls
    xor eax,eax
    ret

.mouse_down:
    mov eax,[lparam]
    mov [mouseParam],eax
    call BeginHexSplitterDrag
    test eax,eax
    jz .status_refresh
    xor eax,eax
    ret

.mouse_move:
    cmp [splitDrag],0
    je .status_refresh
    mov eax,[lparam]
    mov [mouseParam],eax
    call UpdateHexSplitterDrag
    xor eax,eax
    ret

.mouse_up:
    cmp [splitDrag],0
    je .status_refresh
    mov [splitDrag],0
    invoke ReleaseCapture
    call LayoutControls
    cmp [hexDirty],0
    je .mouse_up_done
    call UpdateHexPreview
.mouse_up_done:
    xor eax,eax
    ret

.context_menu:
    mov eax,[wparam]
    cmp eax,[hwndEdit]
    jne .default
    mov eax,[lparam]
    mov [mouseParam],eax
    call ShowEditorContextMenu
    xor eax,eax
    ret

.init_menu_popup:
    call UpdateEditMenuState
    xor eax,eax
    ret

.command:
    mov eax,[wparam]
    shr eax,16
    cmp eax,EN_CHANGE
    jne .cmd_id
    mov eax,[wparam]
    and eax,0FFFFh
    cmp eax,ID_EDIT
    jne .cmd_id
    cmp [loading],0
    jne .change_ignored
    mov [modified],1
    mov [hexDirty],1
    call UpdateTitle
    call UpdateStatus
    xor eax,eax
    ret
.change_ignored:
    xor eax,eax
    ret
.cmd_id:
    mov eax,[wparam]
    and eax,0FFFFh
    cmp eax,IDM_NEW
    je .do_new
    cmp eax,IDM_OPEN
    je .do_open
    cmp eax,IDM_SAVE
    je .do_save
    cmp eax,IDM_SAVEAS
    je .do_saveas
    cmp eax,IDM_FILE_INFO
    je .do_fileinfo
    cmp eax,IDM_FIND
    je .do_find
    cmp eax,IDM_FIND_MORE
    je .do_find_more
    cmp eax,IDM_JUMP_LINECHAR
    je .do_jump
    cmp eax,IDM_DARK
    je .do_dark
    cmp eax,IDM_EXIT
    je .do_exit
    cmp eax,IDM_UNDO
    je .do_undo
    cmp eax,IDM_REDO
    je .do_redo
    cmp eax,IDM_CUT
    je .do_cut
    cmp eax,IDM_COPY
    je .do_copy
    cmp eax,IDM_PASTE
    je .do_paste
    cmp eax,IDM_SELECTALL
    je .do_selectall
    cmp eax,IDM_WORDWRAP
    je .do_wordwrap
    cmp eax,IDM_HILITE_CLEAR
    je .do_hilite_clear
    cmp eax,IDM_HILITE_RED
    je .do_hilite_red
    cmp eax,IDM_HILITE_ORANGE
    je .do_hilite_orange
    cmp eax,IDM_HILITE_YELLOW
    je .do_hilite_yellow
    cmp eax,IDM_HILITE_GREEN
    je .do_hilite_green
    cmp eax,IDM_ZOOM_IN
    je .do_zoom_in
    cmp eax,IDM_ZOOM_OUT
    je .do_zoom_out
    cmp eax,IDM_ZOOM_RESET
    je .do_zoom_reset
    cmp eax,IDM_FONT_DEFAULT
    je .do_font_default
    cmp eax,IDM_FONT_SEGOE
    je .do_font_segoe
    cmp eax,IDM_FONT_GEORGIA
    je .do_font_georgia
    cmp eax,IDM_FONT_CONSOLAS
    je .do_font_consolas
    cmp eax,IDM_ABOUT
    je .do_about
    cmp eax,IDM_HEXVIEW
    je .do_hex_right
    cmp eax,IDM_HEX_LEFT
    je .do_hex_left
    cmp eax,IDM_HEX_RIGHT
    je .do_hex_right
    cmp eax,IDM_HEX_BELOW
    je .do_hex_below
    cmp eax,IDM_HEX_CLOSE
    je .do_hex_close
    cmp eax,IDM_SHA256
    je .do_checksum
    cmp eax,IDM_CHECK_UPDATES
    je .do_check_updates
    xor eax,eax
    ret
.do_new:
    call PromptSaveIfModified
    test eax,eax
    jz .done_command
    call NewFile
    xor eax,eax
    ret
.do_open:
    call PromptSaveIfModified
    test eax,eax
    jz .done_command
    call DoOpenDialog
    xor eax,eax
    ret
.do_save:
    call DoSave
    xor eax,eax
    ret
.do_saveas:
    call DoSaveAsDialog
    xor eax,eax
    ret
.do_fileinfo:
    call ShowFileInfo
    xor eax,eax
    ret
.do_find:
    call ShowFind
    xor eax,eax
    ret
.do_find_more:
    call FindMoreFromSelection
    xor eax,eax
    ret
.do_jump:
    call ShowJumpDialog
    xor eax,eax
    ret
.do_dark:
    xor [darkmode],1
    call ApplyTheme
    xor eax,eax
    ret
.do_undo:
    invoke SendMessage,[hwndEdit],WM_UNDO,0,0
    xor eax,eax
    ret
.do_redo:
    xor eax,eax
    ret
.do_cut:
    invoke SendMessage,[hwndEdit],WM_CUT,0,0
    xor eax,eax
    ret
.do_copy:
    invoke SendMessage,[hwndEdit],WM_COPY,0,0
    xor eax,eax
    ret
.do_paste:
    invoke SendMessage,[hwndEdit],WM_PASTE,0,0
    xor eax,eax
    ret
.do_selectall:
    invoke SendMessage,[hwndEdit],EM_SETSEL,0,-1
    xor eax,eax
    ret
.do_wordwrap:
    xor [wordwrap],1
    call ApplyWordWrap
    xor eax,eax
    ret
.do_hilite_clear:
    call ClearSelectionHighlight
    xor eax,eax
    ret
.do_hilite_red:
    mov [highlightColor],00A0A0FFh
    call ApplySelectionHighlight
    xor eax,eax
    ret
.do_hilite_orange:
    mov [highlightColor],0000C8FFh
    call ApplySelectionHighlight
    xor eax,eax
    ret
.do_hilite_yellow:
    mov [highlightColor],0000FFFFh
    call ApplySelectionHighlight
    xor eax,eax
    ret
.do_hilite_green:
    mov [highlightColor],0080FF80h
    call ApplySelectionHighlight
    xor eax,eax
    ret
.do_zoom_in:
    call ZoomEditorIn
    xor eax,eax
    ret
.do_zoom_out:
    call ZoomEditorOut
    xor eax,eax
    ret
.do_zoom_reset:
    call ZoomEditorReset
    xor eax,eax
    ret
.do_font_default:
    mov [editorFontChoice],FONT_DEFAULT
    call ApplyEditorFontChoice
    xor eax,eax
    ret
.do_font_segoe:
    mov [editorFontChoice],FONT_SEGOE
    call ApplyEditorFontChoice
    xor eax,eax
    ret
.do_font_georgia:
    mov [editorFontChoice],FONT_GEORGIA
    call ApplyEditorFontChoice
    xor eax,eax
    ret
.do_font_consolas:
    mov [editorFontChoice],FONT_CONSOLAS
    call ApplyEditorFontChoice
    xor eax,eax
    ret
.do_hex_left:
    mov [hexView],HEX_LEFT
    call EnsureHexView
    xor eax,eax
    ret
.do_hex_right:
    mov [hexView],HEX_RIGHT
    call EnsureHexView
    xor eax,eax
    ret
.do_hex_below:
    mov [hexView],HEX_BELOW
    call EnsureHexView
    xor eax,eax
    ret
.do_hex_close:
    call CloseHexView
    xor eax,eax
    ret
.do_checksum:
    call ShowFileChecksums
    xor eax,eax
    ret
.do_check_updates:
    call CheckForUpdates
    xor eax,eax
    ret
.do_about:
    invoke MessageBox,[hwnd],aboutTxt,APP_TITLE,MB_OK
    xor eax,eax
    ret
.do_exit:
    invoke SendMessage,[hwnd],WM_CLOSE,0,0
    xor eax,eax
    ret
.done_command:
    xor eax,eax
    ret

.drop:
    call PromptSaveIfModified
    test eax,eax
    jz .drop_cancel
    invoke DragQueryFile,[wparam],0,filePath,260
    invoke DragFinish,[wparam]
    mov esi,filePath
    call OpenFileDocument
    xor eax,eax
    ret
.drop_cancel:
    invoke DragFinish,[wparam]
    xor eax,eax
    ret

.ctlcolor:
    cmp [darkmode],0
    je .default
    invoke SetTextColor,[wparam],0E8E8E8h
    invoke SetBkColor,[wparam],202020h
    mov eax,[hDarkBrush]
    ret
.ctlcolor_static:
    cmp [darkmode],0
    je .default
    invoke SetTextColor,[wparam],0E8E8E8h
    invoke SetBkColor,[wparam],303030h
    mov eax,[hStatBrush]
    ret
.default:
    invoke DefWindowProc,[hwnd],[wmsg],[wparam],[lparam]
    ret

.focus:
    invoke SetFocus,[hwndEdit]
    xor eax,eax
    ret

.findmsg:
    call DoFindNext
    xor eax,eax
    ret

.close:
    call PromptSaveIfModified
    test eax,eax
    jz .close_cancel
    invoke DestroyWindow,[hwnd]
    xor eax,eax
    ret
.close_cancel:
    xor eax,eax
    ret

.destroy:
    invoke KillTimer,[hwnd],TIMER_STATUS_ID
    invoke PostQuitMessage,0
    xor eax,eax
    ret
endp

RegisterJumpClass:
    invoke LoadCursor,0,IDC_ARROW
    mov [wc.hCursor],eax
    mov [wc.style],CS_HREDRAW or CS_VREDRAW
    mov [wc.lpfnWndProc],JumpWndProc
    mov [wc.cbClsExtra],0
    mov [wc.cbWndExtra],0
    mov eax,[hInst]
    mov [wc.hInstance],eax
    mov [wc.hIcon],0
    mov [wc.hbrBackground],COLOR_BTNFACE+1
    mov [wc.lpszMenuName],0
    mov [wc.lpszClassName],JUMP_CLASS
    invoke RegisterClass,wc
    ret

proc JumpWndProc uses ebx esi edi, hwnd,wmsg,wparam,lparam
    mov eax,[wmsg]
    cmp eax,WM_CREATE
    je .create
    cmp eax,WM_COMMAND
    je .command
    cmp eax,WM_CLOSE
    je .close
    cmp eax,WM_DESTROY
    je .destroy
    invoke DefWindowProc,[hwnd],[wmsg],[wparam],[lparam]
    ret
.create:
    invoke CreateWindowEx,0,STATIC_CLASS,jumpLineTxt,WS_CHILD or WS_VISIBLE,12,14,110,20,[hwnd],0,[hInst],0
    invoke CreateWindowEx,WS_EX_CLIENTEDGE,WINEDIT_CLASS,0,WS_CHILD or WS_VISIBLE or WS_TABSTOP or ES_AUTOHSCROLL,130,12,120,22,[hwnd],ID_ED_JUMP_LINE,[hInst],0
    mov [hwndJumpLine],eax
    invoke SendMessage,[hwndJumpLine],WM_SETFONT,[hFont],TRUE
    invoke CreateWindowEx,0,STATIC_CLASS,jumpCharTxt,WS_CHILD or WS_VISIBLE,12,48,130,20,[hwnd],0,[hInst],0
    invoke CreateWindowEx,WS_EX_CLIENTEDGE,WINEDIT_CLASS,0,WS_CHILD or WS_VISIBLE or WS_TABSTOP or ES_AUTOHSCROLL,130,46,120,22,[hwnd],ID_ED_JUMP_CHAR,[hInst],0
    mov [hwndJumpChar],eax
    invoke SendMessage,[hwndJumpChar],WM_SETFONT,[hFont],TRUE
    invoke CreateWindowEx,0,BUTTON_CLASS,jumpGoTxt,WS_CHILD or WS_VISIBLE or WS_TABSTOP,70,86,82,26,[hwnd],ID_BTN_JUMP_GO,[hInst],0
    invoke CreateWindowEx,0,BUTTON_CLASS,jumpCancelTxt,WS_CHILD or WS_VISIBLE or WS_TABSTOP,168,86,82,26,[hwnd],ID_BTN_JUMP_CANCEL,[hInst],0
    invoke SetFocus,[hwndJumpLine]
    xor eax,eax
    ret
.command:
    mov eax,[wparam]
    and eax,0FFFFh
    cmp eax,ID_BTN_JUMP_GO
    je .jump
    cmp eax,ID_BTN_JUMP_CANCEL
    je .close
    xor eax,eax
    ret
.jump:
    call GoToLineCharFromDialog
    test eax,eax
    jnz .close
    xor eax,eax
    ret
.close:
    invoke DestroyWindow,[hwnd]
    xor eax,eax
    ret
.destroy:
    mov [hwndJump],0
    xor eax,eax
    ret
endp

RegisterFileInfoClass:
    invoke LoadCursor,0,IDC_ARROW
    mov [wc.hCursor],eax
    mov [wc.style],CS_HREDRAW or CS_VREDRAW
    mov [wc.lpfnWndProc],FileInfoWndProc
    mov [wc.cbClsExtra],0
    mov [wc.cbWndExtra],0
    mov eax,[hInst]
    mov [wc.hInstance],eax
    mov [wc.hIcon],0
    mov [wc.hbrBackground],COLOR_BTNFACE+1
    mov [wc.lpszMenuName],0
    mov [wc.lpszClassName],FILEINFO_CLASS
    invoke RegisterClass,wc
    ret

proc FileInfoWndProc uses ebx esi edi, hwnd,wmsg,wparam,lparam
    mov eax,[wmsg]
    cmp eax,WM_CREATE
    je .create
    cmp eax,WM_COMMAND
    je .command
    cmp eax,WM_CLOSE
    je .close
    cmp eax,WM_DESTROY
    je .destroy
    invoke DefWindowProc,[hwnd],[wmsg],[wparam],[lparam]
    ret
.create:
    invoke CreateWindowEx,WS_EX_CLIENTEDGE,EDIT_CLASS,0,WS_CHILD or WS_VISIBLE or WS_TABSTOP or WS_VSCROLL or WS_HSCROLL or ES_MULTILINE or ES_AUTOVSCROLL or ES_AUTOHSCROLL or ES_READONLY,12,12,820,365,[hwnd],ID_ED_FILEINFO_TEXT,[hInst],0
    mov [hwndFileInfoText],eax
    invoke SendMessage,[hwndFileInfoText],WM_SETFONT,[hFont],TRUE
    call RefreshFileInfoWindow
    invoke CreateWindowEx,0,BUTTON_CLASS,fileInfoCloseTxt,WS_CHILD or WS_VISIBLE or WS_TABSTOP,732,388,100,26,[hwnd],ID_BTN_FILEINFO_CLOSE,[hInst],0
    invoke SendMessage,eax,WM_SETFONT,[hFont],TRUE
    xor eax,eax
    ret
.command:
    mov eax,[wparam]
    and eax,0FFFFh
    cmp eax,ID_BTN_FILEINFO_CLOSE
    je .close
    xor eax,eax
    ret
.close:
    invoke DestroyWindow,[hwnd]
    xor eax,eax
    ret
.destroy:
    mov [hwndFileInfo],0
    mov [hwndFileInfoText],0
    xor eax,eax
    ret
endp

RegisterChecksumClass:
    invoke LoadCursor,0,IDC_ARROW
    mov [wc.hCursor],eax
    mov [wc.style],CS_HREDRAW or CS_VREDRAW
    mov [wc.lpfnWndProc],ChecksumWndProc
    mov [wc.cbClsExtra],0
    mov [wc.cbWndExtra],0
    mov eax,[hInst]
    mov [wc.hInstance],eax
    mov [wc.hIcon],0
    mov [wc.hbrBackground],COLOR_BTNFACE+1
    mov [wc.lpszMenuName],0
    mov [wc.lpszClassName],CHECKSUM_CLASS
    invoke RegisterClass,wc
    ret

proc ChecksumWndProc uses ebx esi edi, hwnd,wmsg,wparam,lparam
    mov eax,[wmsg]
    cmp eax,WM_CREATE
    je .create
    cmp eax,WM_COMMAND
    je .command
    cmp eax,WM_CLOSE
    je .close
    cmp eax,WM_DESTROY
    je .destroy
    invoke DefWindowProc,[hwnd],[wmsg],[wparam],[lparam]
    ret
.create:
    invoke CreateWindowEx,0,STATIC_CLASS,checksumMd5LabelTxt,WS_CHILD or WS_VISIBLE,12,12,130,20,[hwnd],0,[hInst],0
    invoke CreateWindowEx,WS_EX_CLIENTEDGE,WINEDIT_CLASS,md5Hex,WS_CHILD or WS_VISIBLE or WS_TABSTOP or ES_AUTOHSCROLL or ES_READONLY,150,10,420,22,[hwnd],ID_ED_MD5_ACTUAL,[hInst],0
    mov [hwndMd5Actual],eax
    invoke SendMessage,[hwndMd5Actual],WM_SETFONT,[hFont],TRUE
    invoke CreateWindowEx,0,STATIC_CLASS,checksumExpectedMd5Txt,WS_CHILD or WS_VISIBLE,12,42,150,20,[hwnd],0,[hInst],0
    invoke CreateWindowEx,WS_EX_CLIENTEDGE,WINEDIT_CLASS,0,WS_CHILD or WS_VISIBLE or WS_TABSTOP or ES_AUTOHSCROLL,150,40,420,22,[hwnd],ID_ED_MD5_EXPECTED,[hInst],0
    mov [hwndMd5Expected],eax
    invoke SendMessage,[hwndMd5Expected],WM_SETFONT,[hFont],TRUE
    invoke CreateWindowEx,0,STATIC_CLASS,checksumShaLabelTxt,WS_CHILD or WS_VISIBLE,12,78,130,20,[hwnd],0,[hInst],0
    invoke CreateWindowEx,WS_EX_CLIENTEDGE,WINEDIT_CLASS,shaHex,WS_CHILD or WS_VISIBLE or WS_TABSTOP or ES_AUTOHSCROLL or ES_READONLY,150,76,420,22,[hwnd],ID_ED_SHA_ACTUAL,[hInst],0
    mov [hwndShaActual],eax
    invoke SendMessage,[hwndShaActual],WM_SETFONT,[hFont],TRUE
    invoke CreateWindowEx,0,STATIC_CLASS,checksumExpectedShaTxt,WS_CHILD or WS_VISIBLE,12,108,170,20,[hwnd],0,[hInst],0
    invoke CreateWindowEx,WS_EX_CLIENTEDGE,WINEDIT_CLASS,0,WS_CHILD or WS_VISIBLE or WS_TABSTOP or ES_AUTOHSCROLL,150,106,420,22,[hwnd],ID_ED_SHA_EXPECTED,[hInst],0
    mov [hwndShaExpected],eax
    invoke SendMessage,[hwndShaExpected],WM_SETFONT,[hFont],TRUE
    invoke CreateWindowEx,WS_EX_CLIENTEDGE,EDIT_CLASS,checksumResultReadyTxt,WS_CHILD or WS_VISIBLE or WS_TABSTOP or ES_MULTILINE or ES_AUTOVSCROLL or ES_READONLY,12,142,558,54,[hwnd],ID_ED_CHECKSUM_RESULT,[hInst],0
    mov [hwndChecksumResult],eax
    invoke SendMessage,[hwndChecksumResult],WM_SETFONT,[hFont],TRUE
    invoke CreateWindowEx,0,BUTTON_CLASS,checksumCompareTxt,WS_CHILD or WS_VISIBLE or WS_TABSTOP,384,210,86,26,[hwnd],ID_BTN_CHECKSUM_COMPARE,[hInst],0
    invoke CreateWindowEx,0,BUTTON_CLASS,checksumCloseTxt,WS_CHILD or WS_VISIBLE or WS_TABSTOP,484,210,86,26,[hwnd],ID_BTN_CHECKSUM_CLOSE,[hInst],0
    xor eax,eax
    ret
.command:
    mov eax,[wparam]
    and eax,0FFFFh
    cmp eax,ID_BTN_CHECKSUM_COMPARE
    je .compare
    cmp eax,ID_BTN_CHECKSUM_CLOSE
    je .close
    xor eax,eax
    ret
.compare:
    call CompareExpectedChecksums
    xor eax,eax
    ret
.close:
    invoke DestroyWindow,[hwnd]
    xor eax,eax
    ret
.destroy:
    mov [hwndChecksum],0
    xor eax,eax
    ret
endp

MakeMenu:
    invoke CreateMenu
    mov [hMenuMain],eax
    invoke CreatePopupMenu
    mov [hMenuFile],eax
    invoke CreatePopupMenu
    mov [hMenuEdit],eax
    invoke CreatePopupMenu
    mov [hMenuView],eax
    invoke CreatePopupMenu
    mov [hMenuFont],eax
    invoke CreatePopupMenu
    mov [hMenuTools],eax
    invoke CreatePopupMenu
    mov [hMenuHex],eax
    invoke CreatePopupMenu
    mov [hMenuHelp],eax
    invoke CreatePopupMenu
    mov [hMenuHighlight],eax

    invoke AppendMenu,[hMenuFile],MF_STRING,IDM_NEW,menuNewTxt
    invoke AppendMenu,[hMenuFile],MF_STRING,IDM_OPEN,menuOpenTxt
    invoke AppendMenu,[hMenuFile],MF_STRING,IDM_SAVE,menuSaveTxt
    invoke AppendMenu,[hMenuFile],MF_STRING,IDM_SAVEAS,menuSaveAsTxt
    invoke AppendMenu,[hMenuFile],MF_SEPARATOR,0,0
    invoke AppendMenu,[hMenuFile],MF_STRING,IDM_FILE_INFO,menuFileInfoTxt
    invoke AppendMenu,[hMenuFile],MF_SEPARATOR,0,0
    invoke AppendMenu,[hMenuFile],MF_STRING,IDM_EXIT,menuExitTxt

    invoke AppendMenu,[hMenuEdit],MF_STRING,IDM_UNDO,menuUndoTxt
    invoke AppendMenu,[hMenuEdit],MF_STRING or MF_GRAYED,IDM_REDO,menuRedoTxt
    invoke AppendMenu,[hMenuEdit],MF_SEPARATOR,0,0
    invoke AppendMenu,[hMenuEdit],MF_STRING,IDM_CUT,menuCutTxt
    invoke AppendMenu,[hMenuEdit],MF_STRING,IDM_COPY,menuCopyTxt
    invoke AppendMenu,[hMenuEdit],MF_STRING,IDM_PASTE,menuPasteTxt
    invoke AppendMenu,[hMenuEdit],MF_STRING,IDM_SELECTALL,menuSelectAllTxt
    invoke AppendMenu,[hMenuEdit],MF_SEPARATOR,0,0
    invoke AppendMenu,[hMenuEdit],MF_STRING,IDM_FIND,menuFindTxt
    invoke AppendMenu,[hMenuEdit],MF_STRING or MF_GRAYED,IDM_FIND_MORE,menuFindMoreTxt
    invoke AppendMenu,[hMenuEdit],MF_STRING,IDM_JUMP_LINECHAR,menuJumpTxt
    invoke AppendMenu,[hMenuEdit],MF_STRING or MF_GRAYED,IDM_REPLACE,menuReplaceTxt
    invoke AppendMenu,[hMenuEdit],MF_SEPARATOR,0,0
    invoke AppendMenu,[hMenuHighlight],MF_STRING or MF_GRAYED,IDM_HILITE_CLEAR,menuHighlightClearTxt
    invoke AppendMenu,[hMenuHighlight],MF_SEPARATOR,0,0
    invoke AppendMenu,[hMenuHighlight],MF_STRING or MF_GRAYED,IDM_HILITE_RED,menuHighlightRedTxt
    invoke AppendMenu,[hMenuHighlight],MF_STRING or MF_GRAYED,IDM_HILITE_ORANGE,menuHighlightOrangeTxt
    invoke AppendMenu,[hMenuHighlight],MF_STRING or MF_GRAYED,IDM_HILITE_YELLOW,menuHighlightYellowTxt
    invoke AppendMenu,[hMenuHighlight],MF_STRING or MF_GRAYED,IDM_HILITE_GREEN,menuHighlightGreenTxt
    invoke AppendMenu,[hMenuEdit],MF_POPUP,[hMenuHighlight],menuHighlightTxt

    invoke AppendMenu,[hMenuView],MF_STRING or MF_CHECKED,IDM_DARK,menuDarkTxt
    invoke AppendMenu,[hMenuView],MF_STRING,IDM_WORDWRAP,menuWrapTxt
    invoke AppendMenu,[hMenuView],MF_SEPARATOR,0,0
    invoke AppendMenu,[hMenuView],MF_STRING,IDM_ZOOM_IN,menuZoomInTxt
    invoke AppendMenu,[hMenuView],MF_STRING,IDM_ZOOM_OUT,menuZoomOutTxt
    invoke AppendMenu,[hMenuView],MF_STRING,IDM_ZOOM_RESET,menuZoomResetTxt
    invoke AppendMenu,[hMenuView],MF_SEPARATOR,0,0
    invoke AppendMenu,[hMenuFont],MF_STRING or MF_CHECKED,IDM_FONT_DEFAULT,menuFontDefaultTxt
    invoke AppendMenu,[hMenuFont],MF_STRING,IDM_FONT_SEGOE,menuFontSegoeTxt
    invoke AppendMenu,[hMenuFont],MF_STRING,IDM_FONT_GEORGIA,menuFontGeorgiaTxt
    invoke AppendMenu,[hMenuFont],MF_STRING,IDM_FONT_CONSOLAS,menuFontConsolasTxt
    invoke AppendMenu,[hMenuView],MF_POPUP,[hMenuFont],menuFontTxt

    invoke AppendMenu,[hMenuHex],MF_STRING,IDM_HEX_LEFT,menuHexLeftTxt
    invoke AppendMenu,[hMenuHex],MF_STRING,IDM_HEX_RIGHT,menuHexRightTxt
    invoke AppendMenu,[hMenuHex],MF_STRING,IDM_HEX_BELOW,menuHexBelowTxt
    invoke AppendMenu,[hMenuHex],MF_SEPARATOR,0,0
    invoke AppendMenu,[hMenuHex],MF_STRING,IDM_HEX_CLOSE,menuHexCloseTxt
    invoke AppendMenu,[hMenuTools],MF_POPUP,[hMenuHex],menuHexTxt
    invoke AppendMenu,[hMenuTools],MF_STRING,IDM_SHA256,menuChecksumTxt

    invoke AppendMenu,[hMenuHelp],MF_STRING,IDM_CHECK_UPDATES,menuCheckUpdatesTxt
    invoke AppendMenu,[hMenuHelp],MF_SEPARATOR,0,0
    invoke AppendMenu,[hMenuHelp],MF_STRING,IDM_ABOUT,menuAboutTxt

    invoke AppendMenu,[hMenuMain],MF_POPUP,[hMenuFile],menuFileTxt
    invoke AppendMenu,[hMenuMain],MF_POPUP,[hMenuEdit],menuEditTxt
    invoke AppendMenu,[hMenuMain],MF_POPUP,[hMenuView],menuViewTxt
    invoke AppendMenu,[hMenuMain],MF_POPUP,[hMenuTools],menuToolsTxt
    invoke AppendMenu,[hMenuMain],MF_POPUP,[hMenuHelp],menuHelpTxt
    ret

UpdateEditMenuState:
    call HasEditorSelection
    test eax,eax
    jz .no_selection
    invoke EnableMenuItem,[hMenuMain],IDM_CUT,MF_BYCOMMAND or MF_ENABLED
    invoke EnableMenuItem,[hMenuMain],IDM_COPY,MF_BYCOMMAND or MF_ENABLED
    invoke EnableMenuItem,[hMenuMain],IDM_FIND_MORE,MF_BYCOMMAND or MF_ENABLED
    invoke EnableMenuItem,[hMenuMain],IDM_HILITE_CLEAR,MF_BYCOMMAND or MF_ENABLED
    invoke EnableMenuItem,[hMenuMain],IDM_HILITE_RED,MF_BYCOMMAND or MF_ENABLED
    invoke EnableMenuItem,[hMenuMain],IDM_HILITE_ORANGE,MF_BYCOMMAND or MF_ENABLED
    invoke EnableMenuItem,[hMenuMain],IDM_HILITE_YELLOW,MF_BYCOMMAND or MF_ENABLED
    invoke EnableMenuItem,[hMenuMain],IDM_HILITE_GREEN,MF_BYCOMMAND or MF_ENABLED
    ret
.no_selection:
    invoke EnableMenuItem,[hMenuMain],IDM_CUT,MF_BYCOMMAND or MF_GRAYED
    invoke EnableMenuItem,[hMenuMain],IDM_COPY,MF_BYCOMMAND or MF_GRAYED
    invoke EnableMenuItem,[hMenuMain],IDM_FIND_MORE,MF_BYCOMMAND or MF_GRAYED
    invoke EnableMenuItem,[hMenuMain],IDM_HILITE_CLEAR,MF_BYCOMMAND or MF_GRAYED
    invoke EnableMenuItem,[hMenuMain],IDM_HILITE_RED,MF_BYCOMMAND or MF_GRAYED
    invoke EnableMenuItem,[hMenuMain],IDM_HILITE_ORANGE,MF_BYCOMMAND or MF_GRAYED
    invoke EnableMenuItem,[hMenuMain],IDM_HILITE_YELLOW,MF_BYCOMMAND or MF_GRAYED
    invoke EnableMenuItem,[hMenuMain],IDM_HILITE_GREEN,MF_BYCOMMAND or MF_GRAYED
    ret

HasEditorSelection:
    invoke SendMessage,[hwndEdit],EM_GETSEL,selStart,selEnd
    mov eax,[selEnd]
    cmp eax,[selStart]
    jle .no
    mov eax,1
    ret
.no:
    xor eax,eax
    ret

ShowEditorContextMenu:
    ; Right-click menu is selection-aware. Highlight is omitted entirely when
    ; no text is selected, as requested.
    mov eax,[mouseParam]
    cmp eax,-1
    jne .use_lparam
    invoke GetCursorPos,pt_x
    mov eax,[pt_x]
    mov [ctxX],eax
    mov eax,[pt_y]
    mov [ctxY],eax
    jmp .build
.use_lparam:
    movsx eax,word [mouseParam]
    mov [ctxX],eax
    mov eax,[mouseParam]
    sar eax,16
    mov [ctxY],eax
.build:
    invoke CreatePopupMenu
    mov ebx,eax
    call HasEditorSelection
    test eax,eax
    jz .paste_only
    invoke AppendMenu,ebx,MF_STRING,IDM_CUT,menuCutTxt
    invoke AppendMenu,ebx,MF_STRING,IDM_COPY,menuCopyTxt
    invoke AppendMenu,ebx,MF_STRING,IDM_PASTE,menuPasteTxt
    invoke AppendMenu,ebx,MF_SEPARATOR,0,0
    invoke AppendMenu,ebx,MF_STRING,IDM_FIND_MORE,menuFindMoreTxt
    invoke CreatePopupMenu
    mov edi,eax
    invoke AppendMenu,edi,MF_STRING,IDM_HILITE_CLEAR,menuHighlightClearTxt
    invoke AppendMenu,edi,MF_SEPARATOR,0,0
    invoke AppendMenu,edi,MF_STRING,IDM_HILITE_RED,menuHighlightRedTxt
    invoke AppendMenu,edi,MF_STRING,IDM_HILITE_ORANGE,menuHighlightOrangeTxt
    invoke AppendMenu,edi,MF_STRING,IDM_HILITE_YELLOW,menuHighlightYellowTxt
    invoke AppendMenu,edi,MF_STRING,IDM_HILITE_GREEN,menuHighlightGreenTxt
    invoke AppendMenu,ebx,MF_SEPARATOR,0,0
    invoke AppendMenu,ebx,MF_POPUP,edi,menuHighlightTxt
    jmp .track
.paste_only:
    invoke AppendMenu,ebx,MF_STRING,IDM_PASTE,menuPasteTxt
.track:
    invoke TrackPopupMenu,ebx,TPM_RETURNCMD or TPM_RIGHTBUTTON,[ctxX],[ctxY],0,[hwndMain],0
    test eax,eax
    jz .destroy
    invoke SendMessage,[hwndMain],WM_COMMAND,eax,0
.destroy:
    invoke DestroyMenu,ebx
    ret

CopySelectionToFindBuf:
    ; Copy at most 255 UTF-16 chars from current selection to findBuf.
    call HasEditorSelection
    test eax,eax
    jz .fail
    mov eax,[selEnd]
    sub eax,[selStart]
    cmp eax,255
    jle .len_ok
    mov eax,255
.len_ok:
    mov ecx,[selStart]
    mov [tr_chrg_min],ecx
    add ecx,eax
    mov [tr_chrg_max],ecx
    mov [tr_text],findBuf
    invoke SendMessage,[hwndEdit],EM_GETTEXTRANGE,0,tr_chrg_min
    mov eax,[tr_chrg_max]
    sub eax,[tr_chrg_min]
    mov word [findBuf+eax*2],0
    mov [findNoMoreShown],0
    mov edi,lastFindBuf
    mov word [edi],0
    mov eax,1
    ret
.fail:
    xor eax,eax
    ret

FindMoreFromSelection:
    call CopySelectionToFindBuf
    test eax,eax
    jz .ret
    call ShowFind
    cmp [hFindDlg],0
    je .ret
    invoke SetDlgItemText,[hFindDlg],ID_FIND_DIALOG_TEXT,findBuf
.ret:
    ret

ApplySelectionHighlight:
    call HasEditorSelection
    test eax,eax
    jz .ret
    mov [loading],1
    invoke RtlZeroMemory,charFmt,CHARFORMAT2W_SIZE
    mov dword [charFmt],CHARFORMAT2W_SIZE
    mov dword [charFmt+4],CFM_BACKCOLOR
    mov eax,[highlightColor]
    mov dword [charFmt+96],eax
    invoke SendMessage,[hwndEdit],EM_SETCHARFORMAT,SCF_SELECTION,charFmt
    mov [loading],0
.ret:
    ret

ClearSelectionHighlight:
    call HasEditorSelection
    test eax,eax
    jz .ret
    mov [loading],1
    invoke RtlZeroMemory,charFmt,CHARFORMAT2W_SIZE
    mov dword [charFmt],CHARFORMAT2W_SIZE
    mov dword [charFmt+4],CFM_BACKCOLOR
    mov dword [charFmt+8],CFE_AUTOBACKCOLOR
    invoke SendMessage,[hwndEdit],EM_SETCHARFORMAT,SCF_SELECTION,charFmt
    mov [loading],0
.ret:
    ret



CreateEditorFont:
    mov eax,-16
    sub eax,[editorZoom]
    mov [tmpSize],eax
    cmp [editorFontChoice],FONT_SEGOE
    je .segoe
    cmp [editorFontChoice],FONT_GEORGIA
    je .georgia
    cmp [editorFontChoice],FONT_CONSOLAS
    je .consolas
.default:
    invoke CreateFont, [tmpSize],0,0,0,FW_NORMAL,0,0,0,DEFAULT_CHARSET,OUT_DEFAULT_PRECIS,CLIP_DEFAULT_PRECIS,CLEARTYPE_QUALITY,FIXED_PITCH or FF_MODERN,0
    mov [hEditorFont],eax
    ret
.segoe:
    invoke CreateFont, [tmpSize],0,0,0,FW_NORMAL,0,0,0,DEFAULT_CHARSET,OUT_DEFAULT_PRECIS,CLIP_DEFAULT_PRECIS,CLEARTYPE_QUALITY,DEFAULT_PITCH or FF_DONTCARE,FONT_SEGOE_FACE
    mov [hEditorFont],eax
    ret
.georgia:
    invoke CreateFont, [tmpSize],0,0,0,FW_NORMAL,0,0,0,DEFAULT_CHARSET,OUT_DEFAULT_PRECIS,CLIP_DEFAULT_PRECIS,CLEARTYPE_QUALITY,DEFAULT_PITCH or FF_ROMAN,FONT_GEORGIA_FACE
    mov [hEditorFont],eax
    ret
.consolas:
    invoke CreateFont, [tmpSize],0,0,0,FW_NORMAL,0,0,0,DEFAULT_CHARSET,OUT_DEFAULT_PRECIS,CLIP_DEFAULT_PRECIS,CLEARTYPE_QUALITY,FIXED_PITCH or FF_MODERN,FONT_CONSOLAS_FACE
    mov [hEditorFont],eax
    ret

ZoomEditorIn:
    cmp [editorZoom],18
    jge .ret
    add [editorZoom],2
    call ApplyEditorFontChoice
.ret:
    ret

ZoomEditorOut:
    cmp [editorZoom],-8
    jle .ret
    sub [editorZoom],2
    call ApplyEditorFontChoice
.ret:
    ret

ZoomEditorReset:
    mov [editorZoom],0
    call ApplyEditorFontChoice
    ret

ApplyEditorFontChoice:
    cmp [hEditorFont],0
    je .make
    invoke DeleteObject,[hEditorFont]
    mov [hEditorFont],0
.make:
    call CreateEditorFont
    cmp [hwndEdit],0
    je .checks
    invoke SendMessage,[hwndEdit],WM_SETFONT,[hEditorFont],TRUE
    call ApplyTheme
    call ApplyEditorFontToRichText
    invoke InvalidateRect,[hwndEdit],0,TRUE
.checks:
    invoke CheckMenuItem,[hMenuMain],IDM_FONT_DEFAULT,MF_BYCOMMAND or MF_UNCHECKED
    invoke CheckMenuItem,[hMenuMain],IDM_FONT_SEGOE,MF_BYCOMMAND or MF_UNCHECKED
    invoke CheckMenuItem,[hMenuMain],IDM_FONT_GEORGIA,MF_BYCOMMAND or MF_UNCHECKED
    invoke CheckMenuItem,[hMenuMain],IDM_FONT_CONSOLAS,MF_BYCOMMAND or MF_UNCHECKED
    cmp [editorFontChoice],FONT_SEGOE
    je .check_segoe
    cmp [editorFontChoice],FONT_GEORGIA
    je .check_georgia
    cmp [editorFontChoice],FONT_CONSOLAS
    je .check_consolas
    invoke CheckMenuItem,[hMenuMain],IDM_FONT_DEFAULT,MF_BYCOMMAND or MF_CHECKED
    ret
.check_segoe:
    invoke CheckMenuItem,[hMenuMain],IDM_FONT_SEGOE,MF_BYCOMMAND or MF_CHECKED
    ret
.check_georgia:
    invoke CheckMenuItem,[hMenuMain],IDM_FONT_GEORGIA,MF_BYCOMMAND or MF_CHECKED
    ret
.check_consolas:
    invoke CheckMenuItem,[hMenuMain],IDM_FONT_CONSOLAS,MF_BYCOMMAND or MF_CHECKED
    ret

ApplyEditorFontToRichText:
    ; RichEdit keeps formatting on loaded text. WM_SETFONT updates the control,
    ; but existing runs need an explicit face-name format to visibly change.
    cmp [hwndEdit],0
    je .ret
    invoke RtlZeroMemory,charFmt,CHARFORMATW_SIZE
    mov dword [charFmt],CHARFORMATW_SIZE
    mov dword [charFmt+4],CFM_FACE
    cmp [editorFontChoice],FONT_SEGOE
    je .segoe
    cmp [editorFontChoice],FONT_GEORGIA
    je .georgia
    cmp [editorFontChoice],FONT_CONSOLAS
    je .consolas
    mov esi,FONT_DEFAULT_FACE
    jmp .copy
.segoe:
    mov esi,FONT_SEGOE_FACE
    jmp .copy
.georgia:
    mov esi,FONT_GEORGIA_FACE
    jmp .copy
.consolas:
    mov esi,FONT_CONSOLAS_FACE
.copy:
    mov edi,charFmt+26
    call StrCopyW
    invoke SendMessage,[hwndEdit],EM_SETCHARFORMAT,SCF_DEFAULT,charFmt
    invoke SendMessage,[hwndEdit],EM_SETCHARFORMAT,SCF_ALL,charFmt
.ret:
    ret

CreateEditorControl:
    mov eax,WS_CHILD or WS_VISIBLE or WS_VSCROLL or ES_MULTILINE or ES_AUTOVSCROLL or ES_NOHIDESEL
    cmp [wordwrap],0
    jne .style_ok
    or eax,WS_HSCROLL or ES_AUTOHSCROLL
.style_ok:
    mov [editStyle],eax
    invoke CreateWindowEx,WS_EX_CLIENTEDGE,EDIT_CLASS,0,[editStyle],0,0,0,0,[parentHwnd],ID_EDIT,[hInst],0
    mov [hwndEdit],eax
    invoke SendMessage,[hwndEdit],WM_SETFONT,[hEditorFont],TRUE
    call ApplyEditorFontToRichText
    invoke SendMessage,[hwndEdit],EM_SETMARGINS,EC_LEFTMARGIN,5
    invoke SendMessage,[hwndEdit],EM_LIMITTEXT,0,0
    invoke SendMessage,[hwndEdit],EM_EXLIMITTEXT,0,7FFFFFFFh
    invoke SendMessage,[hwndEdit],EM_SETEVENTMASK,0,ENM_CHANGE
    ret

ApplyWordWrap:
    cmp [wordwrap],0
    je .wrap_off
    invoke CheckMenuItem,[hMenuMain],IDM_WORDWRAP,MF_BYCOMMAND or MF_CHECKED
    jmp .recreate
.wrap_off:
    invoke CheckMenuItem,[hMenuMain],IDM_WORDWRAP,MF_BYCOMMAND or MF_UNCHECKED
.recreate:
    cmp [hwndEdit],0
    je .done

    invoke SendMessage,[hwndEdit],EM_GETSEL,selStart,selEnd
    invoke SendMessage,[hwndEdit],WM_GETTEXTLENGTH,0,0
    mov [tmpLen],eax
    inc eax
    shl eax,1
    invoke GlobalAlloc,GMEM_FIXED or GMEM_ZEROINIT,eax
    test eax,eax
    jz .done
    mov [tmpPtr],eax

    mov eax,[tmpLen]
    inc eax
    invoke SendMessage,[hwndEdit],WM_GETTEXT,eax,[tmpPtr]
    invoke DestroyWindow,[hwndEdit]

    mov eax,[hwndMain]
    mov [parentHwnd],eax
    call CreateEditorControl
    mov [loading],1
    invoke SendMessage,[hwndEdit],WM_SETREDRAW,FALSE,0
    invoke SetWindowText,[hwndEdit],[tmpPtr]
    invoke SendMessage,[hwndEdit],WM_SETREDRAW,TRUE,0
    invoke InvalidateRect,[hwndEdit],0,TRUE
    mov [loading],0
    invoke SendMessage,[hwndEdit],EM_SETSEL,[selStart],[selEnd]
    invoke GlobalFree,[tmpPtr]
    mov [tmpPtr],0

    call LayoutControls
    call ApplyTheme
    invoke ShowWindow,[hwndEdit],SW_SHOW
    call UpdateStatus
    invoke SetFocus,[hwndEdit]
.done:
    ret

NewFile:
    invoke SetWindowText,[hwndEdit],0
    invoke RtlZeroMemory,filePath,260*2
    mov [modified],0
    mov [encoding],ENC_UTF8
    mov [fileBomBytes],3
    mov [lineEnding],EOL_CRLF
    call ClearHashes
    mov [hexDirty],1
    cmp [hwndHex],0
    je .no_hex_clear
    invoke SetWindowText,[hwndHex],0
.no_hex_clear:
    call UpdateTitle
    call UpdateStatus
    ret

PrepareOFN:
    invoke RtlZeroMemory,ofn,sizeof.OPENFILENAME
    mov [ofn.lStructSize],sizeof.OPENFILENAME
    mov eax,[hwndMain]
    mov [ofn.hwndOwner],eax
    mov eax,[hInst]
    mov [ofn.hInstance],eax
    mov [ofn.lpstrFilter],filterTxt
    mov [ofn.lpstrFile],filePath
    mov [ofn.nMaxFile],260
    mov [ofn.Flags],OFN_EXPLORER or OFN_PATHMUSTEXIST or OFN_HIDEREADONLY
    ret

DoOpenDialog:
    call PrepareOFN
    invoke GetOpenFileName,ofn
    test eax,eax
    jz .done
    mov esi,filePath
    call OpenFileDocument
.done:
    ret

OpenCommandLineFile:
    ; beta 6: Explorer file associations and "Open With" pass the selected file
    ; as argv[1]. Parse the Unicode command line so quoted/non-ASCII paths work.
    invoke GetCommandLineW
    invoke CommandLineToArgvW,eax,cmdArgc
    test eax,eax
    jz .done
    mov [cmdArgv],eax
    cmp [cmdArgc],2
    jne .free_argv
    mov ebx,[cmdArgv]
    mov esi,[ebx+4]
    call OpenFileDocument
.free_argv:
    invoke LocalFree,[cmdArgv]
    mov [cmdArgv],0
.done:
    ret

OpenFileDocument:
    ; beta 6: all open entry points converge here. FASM's kernel32 include
    ; already imports WinAPI OpenFile, so the ByteForge OpenFile implementation
    ; uses this project-local name. File->Open and drag/drop already place a
    ; MAX_PATH string in filePath; command-line startup copies argv[1] into the
    ; same buffer before using the existing LoadFile routine.
    cmp esi,filePath
    je .load
    call CopyPathToFilePath
.load:
    call LoadFile
    ret

CopyPathToFilePath:
    ; Copy at most 259 UTF-16 code units into the existing MAX_PATH buffer.
    ; This preserves the original fixed-size filePath contract used elsewhere.
    push ecx
    push edi
    mov edi,filePath
    mov ecx,259
.copy_loop:
    lodsw
    stosw
    test ax,ax
    jz .done
    loop .copy_loop
    mov word [edi],0
.done:
    pop edi
    pop ecx
    ret

DoSave:
    cmp word [filePath],0
    jne .save
    call DoSaveAsDialog
    ret
.save:
    call SaveFile
    ret

DoSaveAsDialog:
    call PrepareOFN
    or [ofn.Flags],OFN_OVERWRITEPROMPT
    invoke GetSaveFileName,ofn
    test eax,eax
    jz .done
    call SaveFile
.done:
    ret

PromptSaveIfModified:
    ; RC2 beta 8: central guard for destructive document changes.
    ; A brand-new empty document can receive an initial EN_CHANGE from RichEdit;
    ; do not ask to save it unless it actually contains text.
    ; Returns EAX=1 to continue, EAX=0 to cancel the caller's action.
    cmp [modified],0
    jne .maybe_empty_new
    mov eax,1
    ret
.maybe_empty_new:
    cmp word [filePath],0
    jne .ask
    invoke SendMessage,[hwndEdit],WM_GETTEXTLENGTH,0,0
    test eax,eax
    jne .ask
    mov [modified],0
    mov eax,1
    ret
.ask:
    invoke MessageBox,[hwndMain],savePromptTxt,APP_TITLE,MB_ICONQUESTION or MB_YESNOCANCEL
    cmp eax,IDCANCEL
    je .cancel
    cmp eax,IDNO
    je .continue
    cmp eax,IDYES
    jne .cancel
    call DoSave
    cmp [modified],0
    jne .cancel
.continue:
    mov eax,1
    ret
.cancel:
    xor eax,eax
    ret

LoadFile:
    ; Use global handles/pointers. Do not keep the file handle in EBX across calls.
    invoke CreateFile,filePath,GENERIC_READ,FILE_SHARE_READ or FILE_SHARE_WRITE,0,OPEN_EXISTING,FILE_ATTRIBUTE_NORMAL,0
    cmp eax,INVALID_HANDLE_VALUE
    je .done
    mov [hFileRead],eax

    invoke GetFileSize,[hFileRead],0
    mov [fileSize],eax
    add eax,8
    invoke GlobalAlloc,GMEM_FIXED or GMEM_ZEROINIT,eax
    test eax,eax
    jz .close_only
    mov [rawPtr],eax

    invoke ReadFile,[hFileRead],[rawPtr],[fileSize],bytesIO,0
    mov eax,[fileSize]
    mov edx,[rawPtr]
    mov byte [edx+eax],0
    mov byte [edx+eax+1],0
    mov byte [edx+eax+2],0
    mov byte [edx+eax+3],0

    invoke KillTimer,[hwndMain],TIMER_STATUS_ID
    mov [loading],1
    invoke SendMessage,[hwndEdit],WM_SETREDRAW,FALSE,0

    mov esi,[rawPtr]
    call DecodeToEdit
    call ApplyTheme
    call ApplyEditorFontToRichText

    invoke SendMessage,[hwndEdit],WM_SETREDRAW,TRUE,0
    invoke InvalidateRect,[hwndEdit],0,TRUE
    mov [loading],0
    invoke SetTimer,[hwndMain],TIMER_STATUS_ID,TIMER_STATUS_MS,0

    invoke GlobalFree,[rawPtr]
    mov [rawPtr],0
    mov [modified],0

.close_only:
    invoke CloseHandle,[hFileRead]
    mov [hFileRead],0
    call ComputeFileHashes
    mov [hexDirty],1
    call UpdateTitle
    call UpdateStatus
    call UpdateHexPreview
.done:
    ret

LowerAsciiW:
    cmp ax,'A'
    jb .ret
    cmp ax,'Z'
    ja .ret
    add ax,32
.ret:
    ret

DecodeToEdit:
    mov [encoding],ENC_UTF8
    mov [fileBomBytes],0
    mov edi,esi
    mov ecx,[fileSize]
    cmp ecx,2
    jb .as_utf8
    cmp word [esi],0FEFFh
    je .utf16le
    cmp word [esi],0FFFEh
    je .utf16be
    cmp ecx,3
    jb .as_utf8
    cmp byte [esi],0EFh
    jne .as_utf8
    cmp byte [esi+1],0BBh
    jne .as_utf8
    cmp byte [esi+2],0BFh
    jne .as_utf8
    add edi,3
    sub ecx,3
    mov [fileBomBytes],3
.as_utf8:
    mov [encoding],ENC_UTF8
    mov [tmpPtr],edi
    mov [tmpLen],ecx
    invoke MultiByteToWideChar,CP_UTF8,0,[tmpPtr],[tmpLen],0,0
    test eax,eax
    jnz .got_chars
    invoke MultiByteToWideChar,CP_ACP,0,[tmpPtr],[tmpLen],0,0
.got_chars:
    test eax,eax
    jz .ret
    mov [tmpSize],eax
    inc eax
    shl eax,1
    invoke GlobalAlloc,GMEM_FIXED or GMEM_ZEROINIT,eax
    test eax,eax
    jz .ret
    mov [tmpPtr],eax
    invoke MultiByteToWideChar,CP_UTF8,0,edi,[tmpLen],[tmpPtr],[tmpSize]
    test eax,eax
    jnz .set
    invoke MultiByteToWideChar,CP_ACP,0,edi,[tmpLen],[tmpPtr],[tmpSize]
.set:
    mov eax,[tmpPtr]
    mov [scanPtr],eax
    call DetectLineEndingWide
    invoke SetWindowText,[hwndEdit],[tmpPtr]
    invoke GlobalFree,[tmpPtr]
.ret:
    ret
.utf16le:
    mov [encoding],ENC_UTF16LE
    mov [fileBomBytes],2
    add esi,2
    mov [scanPtr],esi
    call DetectLineEndingWide
    invoke SetWindowText,[hwndEdit],esi
    ret
.utf16be:
    mov [encoding],ENC_UTF16BE
    mov [fileBomBytes],2
    mov ecx,[fileSize]
    sub ecx,2
    add ecx,2
    mov [tmpLen],ecx
    invoke GlobalAlloc,GMEM_FIXED or GMEM_ZEROINIT,[tmpLen]
    test eax,eax
    jz .ret
    mov edi,eax
    mov [tmpPtr],eax
    add esi,2
    mov ecx,[fileSize]
    sub ecx,2
.be_loop:
    cmp ecx,2
    jb .be_set
    mov ax,[esi]
    xchg al,ah
    mov [edi],ax
    add esi,2
    add edi,2
    sub ecx,2
    jmp .be_loop
.be_set:
    mov word [edi],0
    mov eax,[tmpPtr]
    mov [scanPtr],eax
    call DetectLineEndingWide
    invoke SetWindowText,[hwndEdit],[tmpPtr]
    invoke GlobalFree,[tmpPtr]
    ret

SaveFile:
    ; Robust save path for existing files and new files.
    ; Use global handles/pointers instead of EBX/ESI-owned lifetime.
    invoke SendMessage,[hwndEdit],WM_GETTEXTLENGTH,0,0
    mov [tmpLen],eax                    ; UTF-16 char count, without terminating NUL
    inc eax
    shl eax,1                           ; bytes including terminating NUL
    invoke GlobalAlloc,GMEM_FIXED or GMEM_ZEROINIT,eax
    test eax,eax
    jz .done
    mov [textPtr],eax

    mov eax,[tmpLen]
    inc eax
    invoke SendMessage,[hwndEdit],WM_GETTEXT,eax,[textPtr]

.open_try:
    ; CREATE_ALWAYS truncates an existing file or creates a new one.
    ; FILE_SHARE_READ|FILE_SHARE_WRITE avoids false failures from AV/indexers,
    ; but if we leaked our own read handle before, this still used to fail with 32.
    invoke CreateFile,filePath,GENERIC_WRITE,FILE_SHARE_READ or FILE_SHARE_WRITE,0,CREATE_ALWAYS,FILE_ATTRIBUTE_NORMAL,0
    cmp eax,INVALID_HANDLE_VALUE
    jne .opened

    invoke GetFileAttributes,filePath
    cmp eax,INVALID_FILE_ATTRIBUTES
    je .save_failed
    mov [tmpAttr],eax
    test eax,FILE_ATTRIBUTE_READONLY
    jz .save_failed
    and eax,not FILE_ATTRIBUTE_READONLY
    invoke SetFileAttributes,filePath,eax
    invoke CreateFile,filePath,GENERIC_WRITE,FILE_SHARE_READ or FILE_SHARE_WRITE,0,CREATE_ALWAYS,FILE_ATTRIBUTE_NORMAL,0
    cmp eax,INVALID_HANDLE_VALUE
    je .save_failed

.opened:
    mov [hFileWrite],eax
    cmp [encoding],ENC_UTF16LE
    je .save16le
    cmp [encoding],ENC_UTF16BE
    je .save16be
    call SaveUTF8
    jmp .close_ok

.save16le:
    mov word [statusBuf],0FEFFh
    invoke WriteFile,[hFileWrite],statusBuf,2,bytesIO,0
    mov eax,[tmpLen]
    shl eax,1
    invoke WriteFile,[hFileWrite],[textPtr],eax,bytesIO,0
    jmp .close_ok

.save16be:
    mov word [statusBuf],0FFFEh
    invoke WriteFile,[hFileWrite],statusBuf,2,bytesIO,0
    mov eax,[tmpLen]
    shl eax,1
    invoke GlobalAlloc,GMEM_FIXED or GMEM_ZEROINIT,eax
    test eax,eax
    jz .close_ok
    mov [convPtr],eax
    mov edi,eax
    mov esi,[textPtr]
    mov ecx,[tmpLen]
.be_loop:
    test ecx,ecx
    jz .be_write
    mov ax,[esi]
    xchg al,ah
    mov [edi],ax
    add esi,2
    add edi,2
    dec ecx
    jmp .be_loop
.be_write:
    mov eax,edi
    sub eax,[convPtr]
    invoke WriteFile,[hFileWrite],[convPtr],eax,bytesIO,0
    invoke GlobalFree,[convPtr]
    mov [convPtr],0
    jmp .close_ok

.close_ok:
    invoke CloseHandle,[hFileWrite]
    mov [hFileWrite],0
    mov [modified],0
    call ComputeFileHashes
    mov [hexDirty],1
    call UpdateTitle
    call UpdateStatus
    call UpdateHexPreview
    jmp .free

.save_failed:
    invoke GetLastError
    call FormatSaveError
    invoke MessageBox,[hwndMain],statusBuf,APP_TITLE,MB_ICONERROR or MB_OK
    cmp [hFileWrite],0
    je .free
    invoke CloseHandle,[hFileWrite]
    mov [hFileWrite],0

.free:
    invoke GlobalFree,[textPtr]
    mov [textPtr],0
.done:
    ret

SaveUTF8:
    ; Save current text buffer as UTF-8 with BOM.
    mov [fileBomBytes],3
    mov dword [statusBuf],0BFBBEFh
    invoke WriteFile,[hFileWrite],statusBuf,3,bytesIO,0
    invoke WideCharToMultiByte,CP_UTF8,0,[textPtr],[tmpLen],0,0,0,0
    test eax,eax
    jz .ret
    mov [tmpSize],eax
    invoke GlobalAlloc,GMEM_FIXED or GMEM_ZEROINIT,[tmpSize]
    test eax,eax
    jz .ret
    mov [convPtr],eax
    invoke WideCharToMultiByte,CP_UTF8,0,[textPtr],[tmpLen],[convPtr],[tmpSize],0,0
    mov [tmpOutLen],eax
    invoke WriteFile,[hFileWrite],[convPtr],[tmpOutLen],bytesIO,0
    invoke GlobalFree,[convPtr]
    mov [convPtr],0
.ret:
    ret

ShowFind:
    cmp [hFindDlg],0
    jne .active
    invoke RtlZeroMemory,fr,sizeof.FINDREPLACE
    mov [fr.lStructSize],sizeof.FINDREPLACE
    mov eax,[hwndMain]
    mov [fr.hwndOwner],eax
    mov [fr.lpstrFindWhat],findBuf
    mov [fr.wFindWhatLen],256
    mov [fr.Flags],FR_DOWN
    invoke FindText,fr
    mov [hFindDlg],eax
    ret
.active:
    invoke SetActiveWindow,[hFindDlg]
    ret

ShowJumpDialog:
    cmp [hwndJump],0
    jne .active
    invoke CreateWindowEx,WS_EX_DLGMODALFRAME,JUMP_CLASS,jumpTitleTxt,WS_OVERLAPPED or WS_CAPTION or WS_SYSMENU or WS_VISIBLE,CW_USEDEFAULT,CW_USEDEFAULT,285,155,[hwndMain],0,[hInst],0
    mov [hwndJump],eax
    ret
.active:
    invoke SetActiveWindow,[hwndJump]
    invoke SetFocus,[hwndJumpLine]
    ret

GoToLineCharFromDialog:
    invoke GetWindowText,[hwndJumpLine],jumpLineBuf,32
    mov esi,jumpLineBuf
    call ParseUIntW
    test eax,eax
    jnz .line_ok
    invoke SetFocus,[hwndJumpLine]
    xor eax,eax
    ret
.line_ok:
    dec eax
    mov [lineIdx],eax
    invoke GetWindowText,[hwndJumpChar],jumpCharBuf,32
    cmp word [jumpCharBuf],0
    je .char_default
    mov esi,jumpCharBuf
    call ParseUIntW
    test eax,eax
    jnz .char_ready
    invoke SetFocus,[hwndJumpChar]
    xor eax,eax
    ret
.char_default:
    mov eax,1
.char_ready:
    dec eax
    mov [jumpCharIdx],eax
    call ComputeLogicalJumpSelection
    test eax,eax
    jz .fail
    invoke SendMessage,[hwndEdit],EM_SETSEL,[selStart],[selEnd]
    invoke SendMessage,[hwndEdit],EM_SCROLLCARET,0,0
    invoke SetFocus,[hwndEdit]
    call SyncHexToEditorCaret
    call UpdateStatusAfterJump
    mov eax,1
    ret
.fail:
    xor eax,eax
    ret

ParseUIntW:
    ; ESI = NUL-terminated decimal wide string. EAX = value, or 0 if empty/invalid.
    push ebx
    push ecx
    xor eax,eax
    xor ecx,ecx
.loop:
    xor ebx,ebx
    mov bx,[esi]
    test bx,bx
    jz .done
    cmp bx,'0'
    jb .invalid
    cmp bx,'9'
    ja .invalid
    imul eax,10
    sub bx,'0'
    add eax,ebx
    inc ecx
.next:
    add esi,2
    jmp .loop
.done:
    test ecx,ecx
    jnz .ret
.invalid:
    xor eax,eax
.ret:
    pop ecx
    pop ebx
    ret

ComputeLogicalJumpSelection:
    ; Convert 1-based user line/char to a native RichEdit selection by scanning
    ; logical text lines. This avoids EM_LINEINDEX display-line drift when long
    ; lines wrap beside the hex viewer.
    invoke SendMessage,[hwndEdit],WM_GETTEXTLENGTH,0,0
    mov [tmpLen],eax
    test eax,eax
    jnz .alloc
    mov [lineIdx],0
    mov [jumpCharIdx],0
    mov [selStart],0
    mov [selEnd],0
    mov eax,1
    ret
.alloc:
    inc eax
    shl eax,1
    invoke GlobalAlloc,GMEM_FIXED or GMEM_ZEROINIT,eax
    test eax,eax
    jz .fail
    mov [textPtr],eax
    mov eax,[tmpLen]
    inc eax
    shl eax,1
    mov [getTextExCb],eax
    mov [getTextExFlags],0
    mov [getTextExCodepage],1200
    mov [getTextExDefaultChar],0
    mov [getTextExUsedDefChar],0
    invoke SendMessage,[hwndEdit],EM_GETTEXTEX,getTextExCb,[textPtr]
    mov [tmpLen],eax
    mov esi,[textPtr]
    mov [scanPtr],esi
    mov [hexLineNo],0
    mov [tmpOutLen],0          ; native char index while scanning
    mov [tmpSize],0            ; current logical line start char index
.find_line:
    mov eax,[hexLineNo]
    cmp eax,[lineIdx]
    je .line_found
    mov eax,[tmpOutLen]
    cmp eax,[tmpLen]
    jae .use_last_line
    mov ax,[esi]
    cmp ax,13
    je .newline_cr
    cmp ax,10
    je .newline_lf
    add esi,2
    inc [tmpOutLen]
    jmp .find_line
.newline_cr:
    add esi,2
    inc [tmpOutLen]
    mov eax,[tmpOutLen]
    cmp eax,[tmpLen]
    jae .line_advanced
    cmp word [esi],10
    jne .line_advanced
    add esi,2
    inc [tmpOutLen]
    jmp .line_advanced
.newline_lf:
    add esi,2
    inc [tmpOutLen]
.line_advanced:
    mov eax,[tmpOutLen]
    mov [tmpSize],eax
    mov [scanPtr],esi
    inc [hexLineNo]
    jmp .find_line
.use_last_line:
    mov eax,[hexLineNo]
    mov [lineIdx],eax
.line_found:
    mov esi,[scanPtr]
    mov eax,[tmpSize]
    mov [tmpOutLen],eax
    mov [hexColNo],0
.line_len:
    mov eax,[tmpOutLen]
    cmp eax,[tmpLen]
    jae .clamp_char
    mov ax,[esi]
    cmp ax,13
    je .clamp_char
    cmp ax,10
    je .clamp_char
    add esi,2
    inc [tmpOutLen]
    inc [hexColNo]
    jmp .line_len
.clamp_char:
    mov eax,[hexColNo]
    test eax,eax
    jnz .has_chars
    mov [jumpCharIdx],0
    jmp .set_sel
.has_chars:
    cmp [jumpCharIdx],eax
    jl .set_sel
    dec eax
    mov [jumpCharIdx],eax
.set_sel:
    mov eax,[tmpSize]
    add eax,[jumpCharIdx]
    mov [selStart],eax
    mov [selEnd],eax
    invoke GlobalFree,[textPtr]
    mov [textPtr],0
    mov eax,1
    ret
.fail:
    xor eax,eax
    ret

UpdateStatusAfterJump:
    call SelectStatusLabels
    mov ebx,[lineIdx]
    inc ebx
    mov ecx,[jumpCharIdx]
    inc ecx
    call FormatStatusLine
    invoke SetWindowText,[hwndStatus],statusBuf
    call FormatInfoLine
    invoke SetWindowText,[hwndStatusInfo],statusBuf
    invoke SetWindowText,[hwndStatusVer],VERSION_W
    ret

DoFindNext:
    ; Use RichEdit native Unicode search instead of manual buffer scan.
    ; The old scanner was fragile after switching to RichEdit and could select random chars.
    mov eax,[fr.Flags]
    test eax,FR_DIALOGTERM
    jz .noterm
    mov [hFindDlg],0
    ret
.noterm:
    test eax,FR_FINDNEXT
    jz .ret

    ; Empty search text: do nothing.
    mov esi,findBuf
    call StrLenW
    test eax,eax
    jz .ret
    call UpdateFindNoMoreState

    ; Use the Find dialog direction.
    ; Forward:  search from selection end to document end.
    ; Upward:   search from selection start back to document start.
    invoke SendMessage,[hwndEdit],EM_GETSEL,selStart,selEnd
    mov [ft_text],findBuf
    mov [ft_res_min],0
    mov [ft_res_max],0

    mov eax,[fr.Flags]
    test eax,FR_DOWN
    jz .search_up

.search_down:
    mov eax,[selEnd]
    mov [ft_chrg_min],eax
    invoke SendMessage,[hwndEdit],WM_GETTEXTLENGTH,0,0
    mov [ft_chrg_max],eax
    invoke SendMessage,[hwndEdit],EM_FINDTEXTEXW,FR_DOWN,ft_chrg_min
    jmp .check_result

.search_up:
    mov eax,[selStart]
    mov [ft_chrg_min],eax
    mov [ft_chrg_max],0
    invoke SendMessage,[hwndEdit],EM_FINDTEXTEXW,0,ft_chrg_min

.check_result:
    cmp eax,-1
    jne .found
    cmp [findNoMoreShown],0
    jne .ret
    mov [findNoMoreShown],1
    invoke MessageBox,[hwndMain],findNotFoundTxt,APP_TITLE,MB_OK or MB_ICONINFORMATION
    invoke SetFocus,[hwndEdit]
    ret

.found:
    mov [findNoMoreShown],0
    invoke SendMessage,[hwndEdit],EM_SETSEL,[ft_res_min],[ft_res_max]
    invoke SendMessage,[hwndEdit],EM_SCROLLCARET,0,0
    call HighlightFindInHex
    invoke SetFocus,[hwndEdit]
    call UpdateStatus
.ret:
    ret

UpdateFindNoMoreState:
    ; Reset the one-shot "no more matches" guard whenever the search text changes.
    mov esi,findBuf
    mov edi,lastFindBuf
.compare_loop:
    mov ax,[esi]
    cmp ax,[edi]
    jne .changed
    test ax,ax
    jz .same
    add esi,2
    add edi,2
    jmp .compare_loop
.changed:
    mov [findNoMoreShown],0
    mov esi,findBuf
    mov edi,lastFindBuf
    call StrCopyW
.same:
    ret



; -----------------------------

HighlightFindInHex:
    ; Keep the hex viewer visually aligned with the active editor search hit.
    ; beta 6 sync fix: do not map RichEdit character positions to byte offsets.
    ; RichEdit can normalize text differently from the on-disk byte preview.
    ; Instead, count which editor occurrence was found, then select the same
    ; occurrence in the plain ASCII column of the rendered hex viewer.
    cmp [hwndHex],0
    je .ret
    cmp [hexView],HEX_OFF
    je .ret
    cmp [hexDirty],0
    je .hex_current
    call UpdateHexPreview
.hex_current:
    call CountEditorFindOccurrence
    test eax,eax
    jz .ret
    mov [tmpOutLen],eax
    call FindNthHexAsciiOccurrence
    test eax,eax
    jz .ret
    invoke SendMessage,[hwndHex],EM_SETSEL,[ft_res_min],[ft_res_max]
    invoke SendMessage,[hwndHex],EM_SCROLLCARET,0,0
.ret:
    ret

CountEditorFindOccurrence:
    ; Output:
    ;   EAX = one-based occurrence number of the current editor hit.
    ; The direction does not matter here: after an upward search selects a hit,
    ; the matching hex hit is still the same Nth occurrence from the top.
    mov eax,[ft_res_min]
    mov [hexSelStart],eax
    mov eax,[ft_res_max]
    mov [hexSelEnd],eax
    mov [tmpOutLen],0
    mov [ft_text],findBuf
    mov [ft_chrg_min],0
    invoke SendMessage,[hwndEdit],WM_GETTEXTLENGTH,0,0
    mov [ft_chrg_max],eax
.loop:
    invoke SendMessage,[hwndEdit],EM_FINDTEXTEXW,FR_DOWN,ft_chrg_min
    cmp eax,-1
    je .fail
    inc [tmpOutLen]
    mov eax,[ft_res_min]
    cmp eax,[hexSelStart]
    jne .advance
    mov eax,[hexSelStart]
    mov [ft_res_min],eax
    mov eax,[hexSelEnd]
    mov [ft_res_max],eax
    mov eax,[tmpOutLen]
    ret
.advance:
    mov eax,[ft_res_max]
    cmp eax,[ft_chrg_min]
    ja .got_next_start
    inc eax
.got_next_start:
    cmp eax,[ft_chrg_max]
    ja .fail
    mov [ft_chrg_min],eax
    jmp .loop
.fail:
    xor eax,eax
    ret

FindNthHexAsciiOccurrence:
    ; Output:
    ;   ft_res_min / ft_res_max = selection in the rendered hex viewer.
    ; Searches only the ASCII dump column, not the offset or hex-byte columns.
    mov esi,findBuf
    call StrLenW
    test eax,eax
    jz .fail
    mov [tmpLen],eax

    cmp [hexSourceBuf],0
    jne .source_buffer
    mov esi,HEX_SOURCE_FILE_W
    jmp .got_source
.source_buffer:
    mov esi,HEX_SOURCE_BUFFER_W
.got_source:
    call StrLenW
    mov [hexBaseLen],eax
    mov esi,HEX_HEADER_W
    call StrLenW
    add [hexBaseLen],eax

    invoke SendMessage,[hwndHex],WM_GETTEXTLENGTH,0,0
    mov [hexChars],eax
    inc eax
    shl eax,1
    invoke GlobalAlloc,GMEM_FIXED or GMEM_ZEROINIT,eax
    test eax,eax
    jz .fail
    mov [hexTextPtr],eax
    mov eax,[hexChars]
    inc eax
    invoke SendMessage,[hwndHex],WM_GETTEXT,eax,[hexTextPtr]

.scan_init:
    mov [hexLineNo],0
.line_loop:
    mov eax,[hexLineNo]
    mov ecx,[hexBytesPerLine]
    mul ecx
    cmp eax,[hexBytes]
    jae .not_found
    mov ebx,[hexBytes]
    sub ebx,eax
    cmp ebx,[hexBytesPerLine]
    jbe .line_bytes_ready
    mov ebx,[hexBytesPerLine]
.line_bytes_ready:
    mov [hexLineBytes],ebx
    mov [hexColNo],0
.col_loop:
    mov eax,[hexColNo]
    cmp eax,[hexLineBytes]
    jae .next_line
    mov ecx,[hexLineBytes]
    sub ecx,eax
    cmp ecx,[tmpLen]
    jb .next_line

    mov eax,[hexLineNo]
    mov ecx,[hexLineChars]
    mul ecx
    add eax,[hexBaseLen]
    add eax,[hexAsciiCol]
    add eax,[hexColNo]
    mov [ft_res_min],eax
    mov esi,[hexTextPtr]
    lea esi,[esi+eax*2]
    mov edi,findBuf
    mov ecx,[tmpLen]
    call MatchFindAtHexAscii
    test eax,eax
    jz .next_col
    dec [tmpOutLen]
    jnz .next_col
    ; Convert the found ASCII-column line/column to a RichEdit selection.
    ; RichEdit normalizes CR/LF internally, so absolute offsets calculated
    ; from the generated text buffer drift as the line number grows. Use
    ; EM_LINEINDEX for the actual line start, then add the fixed ASCII column.
    mov eax,[hexLineNo]
    add eax,2                       ; source banner + header precede data
    invoke SendMessage,[hwndHex],EM_LINEINDEX,eax,0
    add eax,[hexAsciiCol]           ; ASCII column within a hex dump line
    add eax,[hexColNo]
    mov [ft_res_min],eax
    add eax,[tmpLen]
    mov [ft_res_max],eax
    invoke GlobalFree,[hexTextPtr]
    mov [hexTextPtr],0
    mov eax,1
    ret
.next_col:
    inc [hexColNo]
    jmp .col_loop
.next_line:
    inc [hexLineNo]
    jmp .line_loop
.not_found:
    invoke GlobalFree,[hexTextPtr]
    mov [hexTextPtr],0
.fail:
    xor eax,eax
    ret

MatchFindAtHexAscii:
    ; ESI = hex viewer ASCII-column text, EDI = findBuf, ECX = chars.
    ; Honors the Find dialog's match-case flag for ASCII text.
    test ecx,ecx
    jz .match
.loop:
    mov ax,[esi]
    mov bx,[edi]
    test [fr.Flags],FR_MATCHCASE
    jnz .compare
    cmp ax,'A'
    jb .lower_b
    cmp ax,'Z'
    ja .lower_b
    add ax,32
.lower_b:
    cmp bx,'A'
    jb .compare
    cmp bx,'Z'
    ja .compare
    add bx,32
.compare:
    cmp ax,bx
    jne .no_match
    add esi,2
    add edi,2
    dec ecx
    jnz .loop
.match:
    mov eax,1
    ret
.no_match:
    xor eax,eax
    ret

SyncHexToEditorCaret:
    ; Keep hex viewer near a direct line/char jump in the main editor.
    cmp [hwndHex],0
    je .ret
    cmp [hexView],HEX_OFF
    je .ret
    cmp [hexDirty],0
    je .hex_current
    call UpdateHexPreview
.hex_current:
    call ComputeCaretByteOffsetForHex
    test eax,eax
    jz .ret
    call SelectHexByteOffset
.ret:
    ret

ComputeCaretByteOffsetForHex:
    ; Output hexSelStart = byte offset in the current hex preview source.
    ; UTF-16 disk files map exactly. UTF-8/current-buffer previews use the
    ; editor text prefix converted to UTF-8, matching ByteForge's buffer view.
    cmp word [filePath],0
    je .utf8_buffer
    cmp [modified],0
    jne .utf8_buffer
    cmp [encoding],ENC_UTF16LE
    je .utf16
    cmp [encoding],ENC_UTF16BE
    je .utf16
    call ComputeFileByteOffsetByLineChar
    ret
.utf8_buffer:
    ; Buffer previews are always rendered as UTF-8 with BOM, independent of
    ; the source file's original BOM, so the caret byte offset starts at 3.
    mov ebx,3
    jmp .utf8_map
.utf16:
    mov eax,[selStart]
    shl eax,1
    add eax,[fileBomBytes]
    mov [hexSelStart],eax
    mov eax,1
    ret
.utf8_map:
    cmp [selStart],0
    jne .alloc
    mov [hexSelStart],ebx
    mov eax,1
    ret
.alloc:
    mov eax,[selStart]
    inc eax
    shl eax,1
    invoke GlobalAlloc,GMEM_FIXED or GMEM_ZEROINIT,eax
    test eax,eax
    jz .fail
    mov [textPtr],eax
    mov eax,[selStart]
    inc eax
    invoke SendMessage,[hwndEdit],WM_GETTEXT,eax,[textPtr]
    invoke WideCharToMultiByte,CP_UTF8,0,[textPtr],[selStart],0,0,0,0
    add eax,ebx
    mov [hexSelStart],eax
    invoke GlobalFree,[textPtr]
    mov [textPtr],0
    mov eax,1
    ret
.fail:
    xor eax,eax
    ret

ComputeFileByteOffsetByLineChar:
    ; For unchanged disk files, scan the same bytes used by the hex preview.
    ; This avoids RichEdit CR/LF normalization drift after Jump to line/char.
    invoke CreateFile,filePath,GENERIC_READ,FILE_SHARE_READ or FILE_SHARE_WRITE,0,OPEN_EXISTING,FILE_ATTRIBUTE_NORMAL,0
    cmp eax,INVALID_HANDLE_VALUE
    je .fail
    mov [hFileRead],eax
    invoke GlobalAlloc,GMEM_FIXED or GMEM_ZEROINIT,HEX_PREVIEW_BYTES
    test eax,eax
    jz .close_fail
    mov [hexBytePtr],eax
    invoke ReadFile,[hFileRead],[hexBytePtr],HEX_PREVIEW_BYTES,bytesIO,0
    mov eax,[bytesIO]
    mov [hexBytes],eax
    mov esi,[hexBytePtr]
    add esi,[fileBomBytes]
    mov eax,[fileBomBytes]
    mov [tmpLen],eax              ; byte offset in preview/file
    mov eax,[bytesIO]
    sub eax,[fileBomBytes]
    mov [tmpSize],eax             ; bytes left
    mov [hexLineNo],0
    mov [hexColNo],0
.scan:
    mov eax,[hexLineNo]
    cmp eax,[lineIdx]
    jne .before_target
    mov eax,[hexColNo]
    cmp eax,[jumpCharIdx]
    jae .found
    cmp [tmpSize],0
    je .found
    mov al,[esi]
    cmp al,13
    je .found
    cmp al,10
    je .found
    call Utf8CharByteLen
    add esi,eax
    add [tmpLen],eax
    inc [hexColNo]
    sub [tmpSize],eax
    jmp .scan
.before_target:
    cmp [tmpSize],0
    je .found
    mov al,[esi]
    inc esi
    inc [tmpLen]
    dec [tmpSize]
    cmp al,13
    je .cr
    cmp al,10
    je .lf
    jmp .scan
.cr:
    inc [hexLineNo]
    mov [hexColNo],0
    cmp [tmpSize],0
    je .scan
    cmp byte [esi],10
    jne .scan
    inc esi
    inc [tmpLen]
    dec [tmpSize]
    jmp .scan
.lf:
    inc [hexLineNo]
    mov [hexColNo],0
    jmp .scan
.found:
    mov eax,[tmpLen]
    mov [hexSelStart],eax
    invoke GlobalFree,[hexBytePtr]
    mov [hexBytePtr],0
    invoke CloseHandle,[hFileRead]
    mov [hFileRead],0
    mov eax,1
    ret
.close_fail:
    invoke CloseHandle,[hFileRead]
    mov [hFileRead],0
.fail:
    xor eax,eax
    ret

Utf8CharByteLen:
    ; AL = first byte, tmpSize = bytes available. EAX = byte length for one
    ; UTF-8 character. Invalid/incomplete sequences safely count as 1 byte.
    push ebx
    movzx ebx,al
    mov eax,1
    test bl,80h
    jz .done
    mov eax,2
    cmp bl,0C0h
    jb .invalid
    cmp bl,0E0h
    jb .check
    mov eax,3
    cmp bl,0F0h
    jb .check
    mov eax,4
    cmp bl,0F8h
    jb .check
.invalid:
    mov eax,1
    jmp .done
.check:
    cmp [tmpSize],eax
    jae .done
    mov eax,1
.done:
    pop ebx
    ret

SelectHexByteOffset:
    mov eax,[hexSelStart]
    cmp eax,[hexBytes]
    jb .in_range
    xor eax,eax
    ret
.in_range:
    xor edx,edx
    mov ecx,[hexBytesPerLine]
    div ecx
    mov [hexLineNo],eax
    mov [hexColNo],edx
    mov eax,[hexLineNo]
    mov ecx,[hexBytesPerLine]
    mul ecx
    mov edi,hexOffsetNeedle
    call AppendHexDword8
    mov [hx_text],hexOffsetNeedle
    mov [hx_chrg_min],0
    invoke SendMessage,[hwndHex],WM_GETTEXTLENGTH,0,0
    mov [hx_chrg_max],eax
    invoke SendMessage,[hwndHex],EM_FINDTEXTEXW,0,hx_chrg_min
    cmp eax,-1
    je .fallback_line
    mov eax,[hx_res_min]
    jmp .line_start_ready
.fallback_line:
    mov eax,[hexLineNo]
    add eax,2
    invoke SendMessage,[hwndHex],EM_LINEINDEX,eax,0
    cmp eax,-1
    je .fail
.line_start_ready:
    ; Select the actual hex byte cell, not the ASCII shadow column. The offset
    ; search above makes this stable even if RichEdit line indexing changes.
    add eax,11                       ; "00000000 | "
    mov ecx,[hexColNo]
    imul ecx,3
    add eax,ecx
    mov [hexSelStart],eax
    add eax,2
    mov [hexSelEnd],eax
    invoke SendMessage,[hwndHex],EM_SETSEL,[hexSelStart],[hexSelEnd]
    invoke SendMessage,[hwndHex],EM_SCROLLCARET,0,0
    mov eax,1
    ret
.fail:
    xor eax,eax
    ret

; Hex viewer and checksums
; -----------------------------

LayoutControls:
    ; clientW/clientH are content area excluding status bar.
    mov [layoutRedraw],TRUE
    cmp [splitDrag],0
    je .redraw_ready
    mov [layoutRedraw],FALSE
.redraw_ready:
    cmp [hwndHex],0
    je .no_hex
    cmp [hexView],HEX_OFF
    je .no_hex
    cmp [hexView],HEX_LEFT
    je .left
    cmp [hexView],HEX_RIGHT
    je .right
    cmp [hexView],HEX_BELOW
    je .below
.no_hex:
    mov [paneX],0
    mov [paneY],0
    mov eax,[clientW]
    mov [paneW],eax
    mov eax,[clientH]
    mov [paneH],eax
    call MoveTextPane
    jmp .status
.left:
    call ClampHexSplitW
    mov eax,[hexSplitW]
    sub eax,3
    mov [tmpSize],eax
    call SetHexBytesPerLine
    invoke MoveWindow,[hwndHex],0,0,[tmpSize],[clientH],[layoutRedraw]
    mov eax,[hexSplitW]
    add eax,3
    mov [tmpPtr],eax
    mov eax,[clientW]
    sub eax,[tmpPtr]
    mov eax,[tmpPtr]
    mov [paneX],eax
    mov [paneY],0
    mov [paneW],eax
    mov eax,[clientH]
    mov [paneH],eax
    call MoveTextPane
    jmp .status
.right:
    call ClampHexSplitW
    mov eax,[clientW]
    sub eax,[hexSplitW]
    sub eax,3
    mov [tmpSize],eax
    mov [paneX],0
    mov [paneY],0
    mov eax,[tmpSize]
    mov [paneW],eax
    mov eax,[clientH]
    mov [paneH],eax
    call MoveTextPane
    mov eax,[clientW]
    sub eax,[hexSplitW]
    add eax,3
    mov [tmpPtr],eax
    mov eax,[hexSplitW]
    sub eax,3
    call SetHexBytesPerLine
    mov eax,[hexSplitW]
    sub eax,3
    invoke MoveWindow,[hwndHex],[tmpPtr],0,eax,[clientH],[layoutRedraw]
    jmp .status
.below:
    call ClampHexSplitH
    mov eax,[clientH]
    sub eax,[hexSplitH]
    sub eax,3
    mov [paneX],0
    mov [paneY],0
    mov [paneH],eax
    mov eax,[clientW]
    mov [paneW],eax
    call MoveTextPane
    mov eax,[clientH]
    sub eax,[hexSplitH]
    add eax,3
    mov [tmpPtr],eax
    mov eax,[hexSplitH]
    sub eax,3
    mov eax,[clientW]
    call SetHexBytesPerLine
    mov eax,[hexSplitH]
    sub eax,3
    invoke MoveWindow,[hwndHex],0,[tmpPtr],[clientW],eax,[layoutRedraw]
.status:
    mov eax,[clientW]
    sub eax,[leftW]
    sub eax,[rightW]
    cmp eax,40
    jge .size_mid_ok
    mov eax,40
.size_mid_ok:
    mov [midW],eax
    mov eax,[clientW]
    sub eax,[rightW]
    mov [midX],eax
    invoke MoveWindow,[hwndStatus],0,[clientH],[leftW],STATUS_H,[layoutRedraw]
    invoke MoveWindow,[hwndStatusInfo],[leftW],[clientH],[midW],STATUS_H,[layoutRedraw]
    invoke MoveWindow,[hwndStatusVer],[midX],[clientH],[rightW],STATUS_H,[layoutRedraw]
    ret

MoveTextPane:
    invoke MoveWindow,[hwndEdit],[paneX],[paneY],[paneW],[paneH],[layoutRedraw]
    ret

SetHexBytesPerLine:
    ; EAX = current hex pane width in pixels. Convert to a practical number
    ; of bytes per row for the monospace dump so wide panes use their space.
    push ebx
    push edx
    sub eax,15                   ; compact 15px right margin
    cmp eax,160
    jge .width_ok
    mov eax,160
.width_ok:
    mov ebx,9                    ; conservative Courier New char width
    xor edx,edx
    div ebx
    cmp eax,76
    jge .calc
    mov eax,76
.calc:
    sub eax,18                   ; line overhead before 4 chars per byte
    shr eax,2
    and eax,not 3                ; stable multiples of 4 while resizing
    cmp eax,16
    jge .min_ok
    mov eax,16
.min_ok:
    cmp eax,64
    jle .max_ok
    mov eax,64
.max_ok:
    cmp eax,[hexBytesPerLine]
    je .done
    mov [hexBytesPerLine],eax
    mov ebx,eax
    imul ebx,3
    add ebx,14
    mov [hexAsciiCol],ebx
    mov ebx,eax
    shl ebx,2
    add ebx,16
    mov [hexLineChars],ebx
    mov [hexDirty],1
.done:
    pop edx
    pop ebx
    ret

ClampHexSplitW:
    cmp [hexSplitW],180
    jge .min_ok
    mov [hexSplitW],180
.min_ok:
    mov eax,[clientW]
    sub eax,180
    cmp eax,180
    jge .max_ready
    mov eax,[clientW]
    shr eax,1
.max_ready:
    cmp [hexSplitW],eax
    jle .done
    mov [hexSplitW],eax
.done:
    ret

ClampHexSplitH:
    cmp [hexSplitH],120
    jge .min_ok
    mov [hexSplitH],120
.min_ok:
    mov eax,[clientH]
    sub eax,120
    cmp eax,120
    jge .max_ready
    mov eax,[clientH]
    shr eax,1
.max_ready:
    cmp [hexSplitH],eax
    jle .done
    mov [hexSplitH],eax
.done:
    ret

BeginHexSplitterDrag:
    xor eax,eax
    cmp [hwndHex],0
    je .ret
    cmp [hexView],HEX_OFF
    je .ret
    mov edx,[mouseParam]
    and edx,0FFFFh              ; x
    mov ecx,[mouseParam]
    shr ecx,16                  ; y
    cmp [hexView],HEX_LEFT
    je .left
    cmp [hexView],HEX_RIGHT
    je .right
    cmp [hexView],HEX_BELOW
    je .below
    jmp .ret
.left:
    call ClampHexSplitW
    mov eax,[hexSplitW]
    sub eax,16
    cmp edx,eax
    jl .nohit
    mov eax,[hexSplitW]
    add eax,16
    cmp edx,eax
    jg .nohit
    mov [splitDrag],HEX_LEFT
    invoke SetCapture,[hwndMain]
    mov eax,1
    ret
.right:
    call ClampHexSplitW
    mov eax,[clientW]
    sub eax,[hexSplitW]
    sub eax,16
    cmp edx,eax
    jl .nohit
    mov eax,[clientW]
    sub eax,[hexSplitW]
    add eax,16
    cmp edx,eax
    jg .nohit
    mov [splitDrag],HEX_RIGHT
    invoke SetCapture,[hwndMain]
    mov eax,1
    ret
.below:
    call ClampHexSplitH
    mov eax,[clientH]
    sub eax,[hexSplitH]
    sub eax,16
    cmp ecx,eax
    jl .nohit
    mov eax,[clientH]
    sub eax,[hexSplitH]
    add eax,16
    cmp ecx,eax
    jg .nohit
    mov [splitDrag],HEX_BELOW
    invoke SetCapture,[hwndMain]
    mov eax,1
    ret
.nohit:
    xor eax,eax
.ret:
    ret

UpdateHexSplitterDrag:
    mov edx,[mouseParam]
    and edx,0FFFFh              ; x
    mov ecx,[mouseParam]
    shr ecx,16                  ; y
    cmp [splitDrag],HEX_LEFT
    je .left
    cmp [splitDrag],HEX_RIGHT
    je .right
    cmp [splitDrag],HEX_BELOW
    je .below
    ret
.left:
    mov [hexSplitW],edx
    call ClampHexSplitW
    call LayoutControls
    ret
.right:
    mov eax,[clientW]
    sub eax,edx
    mov [hexSplitW],eax
    call ClampHexSplitW
    call LayoutControls
    ret
.below:
    mov eax,[clientH]
    sub eax,ecx
    mov [hexSplitH],eax
    call ClampHexSplitH
    call LayoutControls
    ret

EnsureHexView:
    cmp [hwndHex],0
    jne .exists
    invoke CreateWindowEx,WS_EX_CLIENTEDGE,EDIT_CLASS,0,WS_CHILD or WS_VISIBLE or WS_VSCROLL or WS_HSCROLL or ES_MULTILINE or ES_AUTOVSCROLL or ES_AUTOHSCROLL or ES_READONLY,0,0,0,0,[hwndMain],ID_HEX,[hInst],0
    mov [hwndHex],eax
    invoke SendMessage,[hwndHex],WM_SETFONT,[hHexFont],TRUE
    invoke SendMessage,[hwndHex],EM_HIDESELECTION,FALSE,0
    invoke SendMessage,[hwndHex],EM_SETBKGNDCOLOR,0,202020h
.exists:
    invoke ShowWindow,[hwndHex],SW_SHOW
    invoke CheckMenuItem,[hMenuMain],IDM_HEX_LEFT,MF_BYCOMMAND or MF_UNCHECKED
    invoke CheckMenuItem,[hMenuMain],IDM_HEX_RIGHT,MF_BYCOMMAND or MF_UNCHECKED
    invoke CheckMenuItem,[hMenuMain],IDM_HEX_BELOW,MF_BYCOMMAND or MF_UNCHECKED
    cmp [hexView],HEX_LEFT
    jne .not_left
    invoke CheckMenuItem,[hMenuMain],IDM_HEX_LEFT,MF_BYCOMMAND or MF_CHECKED
.not_left:
    cmp [hexView],HEX_RIGHT
    jne .not_right
    invoke CheckMenuItem,[hMenuMain],IDM_HEX_RIGHT,MF_BYCOMMAND or MF_CHECKED
.not_right:
    cmp [hexView],HEX_BELOW
    jne .not_below
    invoke CheckMenuItem,[hMenuMain],IDM_HEX_BELOW,MF_BYCOMMAND or MF_CHECKED
.not_below:
    call LayoutControls
    call ApplyTheme
    mov [hexDirty],1
    call UpdateHexPreview
    ret

CloseHexView:
    mov [hexView],HEX_OFF
    invoke CheckMenuItem,[hMenuMain],IDM_HEX_LEFT,MF_BYCOMMAND or MF_UNCHECKED
    invoke CheckMenuItem,[hMenuMain],IDM_HEX_RIGHT,MF_BYCOMMAND or MF_UNCHECKED
    invoke CheckMenuItem,[hMenuMain],IDM_HEX_BELOW,MF_BYCOMMAND or MF_UNCHECKED
    cmp [hwndHex],0
    je .layout
    invoke DestroyWindow,[hwndHex]
    mov [hwndHex],0
.layout:
    call LayoutControls
    ret

UpdateHexPreview:
    cmp [hwndHex],0
    je .ret
    cmp [hexView],HEX_OFF
    je .ret
    mov [hexDirty],0

    ; If an existing file is open and unchanged, show exact bytes from disk.
    ; Otherwise show the current editor buffer as it would be inspected now.
    cmp word [filePath],0
    je .buffer
    cmp [modified],0
    jne .buffer
    call UpdateHexPreviewFromFile
    ret
.buffer:
    call UpdateHexPreviewFromBuffer
.ret:
    ret

UpdateHexPreviewFromFile:
    mov [hexSourceBuf],0
    invoke CreateFile,filePath,GENERIC_READ,FILE_SHARE_READ or FILE_SHARE_WRITE,0,OPEN_EXISTING,FILE_ATTRIBUTE_NORMAL,0
    cmp eax,INVALID_HANDLE_VALUE
    je .ret
    mov [hFileRead],eax
    invoke GlobalAlloc,GMEM_FIXED or GMEM_ZEROINIT,HEX_PREVIEW_BYTES
    test eax,eax
    jz .close
    mov [hexBytePtr],eax
    invoke ReadFile,[hFileRead],[hexBytePtr],HEX_PREVIEW_BYTES,bytesIO,0
    mov eax,[bytesIO]
    mov [hexBytes],eax
    invoke GlobalAlloc,GMEM_FIXED or GMEM_ZEROINIT,HEX_OUT_BYTES
    test eax,eax
    jz .free_bytes
    mov [hexOutPtr],eax
    call BuildHexOutput
    invoke SendMessage,[hwndHex],WM_SETREDRAW,FALSE,0
    invoke SetWindowText,[hwndHex],[hexOutPtr]
    invoke SendMessage,[hwndHex],WM_SETREDRAW,TRUE,0
    invoke InvalidateRect,[hwndHex],0,TRUE
    invoke GlobalFree,[hexOutPtr]
    mov [hexOutPtr],0
.free_bytes:
    invoke GlobalFree,[hexBytePtr]
    mov [hexBytePtr],0
.close:
    invoke CloseHandle,[hFileRead]
    mov [hFileRead],0
.ret:
    ret

UpdateHexPreviewFromBuffer:
    mov [hexSourceBuf],1
    mov eax,HEX_PREVIEW_CHARS
    shl eax,1
    invoke GlobalAlloc,GMEM_FIXED or GMEM_ZEROINIT,eax
    test eax,eax
    jz .ret
    mov [hexTextPtr],eax
    invoke SendMessage,[hwndEdit],WM_GETTEXT,HEX_PREVIEW_CHARS,[hexTextPtr]
    mov [hexChars],eax
    test eax,eax
    jnz .has_text

    ; Empty editor buffer: clear the hex pane instead of leaving stale preview.
    invoke SendMessage,[hwndHex],WM_SETREDRAW,FALSE,0
    invoke SetWindowText,[hwndHex],0
    invoke SendMessage,[hwndHex],WM_SETREDRAW,TRUE,0
    invoke InvalidateRect,[hwndHex],0,TRUE
    jmp .free_text

.has_text:
    ; For buffer preview, encode as UTF-8 with BOM. This matches ByteForge's
    ; default new-file save behavior and keeps the preview byte-oriented.
    invoke WideCharToMultiByte,CP_UTF8,0,[hexTextPtr],[hexChars],0,0,0,0
    test eax,eax
    jz .free_text
    add eax,3
    mov [hexBytes],eax
    invoke GlobalAlloc,GMEM_FIXED or GMEM_ZEROINIT,[hexBytes]
    test eax,eax
    jz .free_text
    mov [hexBytePtr],eax
    mov byte [eax],0EFh
    mov byte [eax+1],0BBh
    mov byte [eax+2],0BFh
    mov edx,[hexBytePtr]
    add edx,3
    mov eax,[hexBytes]
    sub eax,3
    invoke WideCharToMultiByte,CP_UTF8,0,[hexTextPtr],[hexChars],edx,eax,0,0
    add eax,3
    mov [hexBytes],eax
    invoke GlobalAlloc,GMEM_FIXED or GMEM_ZEROINIT,HEX_OUT_BYTES
    test eax,eax
    jz .free_bytes
    mov [hexOutPtr],eax
    call BuildHexOutput
    invoke SendMessage,[hwndHex],WM_SETREDRAW,FALSE,0
    invoke SetWindowText,[hwndHex],[hexOutPtr]
    invoke SendMessage,[hwndHex],WM_SETREDRAW,TRUE,0
    invoke InvalidateRect,[hwndHex],0,TRUE
    invoke GlobalFree,[hexOutPtr]
    mov [hexOutPtr],0
.free_bytes:
    invoke GlobalFree,[hexBytePtr]
    mov [hexBytePtr],0
.free_text:
    invoke GlobalFree,[hexTextPtr]
    mov [hexTextPtr],0
.ret:
    ret

BuildHexOutput:
    mov edi,[hexOutPtr]
    cmp [hexSourceBuf],0
    jne .source_buffer
    mov esi,HEX_SOURCE_FILE_W
    jmp .source_copy
.source_buffer:
    mov esi,HEX_SOURCE_BUFFER_W
.source_copy:
    call StrCopyW
    mov esi,HEX_HEADER_W
    call StrCatW
    mov esi,[hexBytePtr]
    mov ecx,[hexBytes]
    xor ebx,ebx
.line:
    test ecx,ecx
    jz .done
    mov eax,ebx
    call AppendHexDword8
    mov ax,' '
    call AppendCharW
    mov ax,'|'
    call AppendCharW
    mov ax,' '
    call AppendCharW
    mov [tmpPtr],esi
    mov [hexLineBytes],0
    xor edx,edx
.hex_loop:
    cmp edx,[hexBytesPerLine]
    jae .ascii_part
    test ecx,ecx
    jz .hex_pad
    mov al,[esi]
    call AppendHexByteW
    inc esi
    inc ebx
    dec ecx
    inc [hexLineBytes]
    inc edx
    jmp .hex_loop
.hex_pad:
    mov ax,' '
    call AppendCharW
    call AppendCharW
    call AppendCharW
    inc edx
    jmp .hex_loop
.ascii_part:
    mov ax,' '
    call AppendCharW
    mov ax,'|'
    call AppendCharW
    mov ax,' '
    call AppendCharW
    mov esi,[tmpPtr]
    mov edx,[hexLineBytes]
.ascii_loop:
    test edx,edx
    jz .newline
    mov al,[esi]
    cmp al,32
    jb .dot
    cmp al,126
    ja .dot
    xor ah,ah
    jmp .putc
.dot:
    mov ax,'.'
.putc:
    call AppendCharW
    inc esi
    dec edx
    jmp .ascii_loop
.newline:
    mov ax,13
    call AppendCharW
    mov ax,10
    call AppendCharW
    jmp .line
.done:
    mov word [edi],0
    ret

AppendCharW:
    mov [edi],ax
    add edi,2
    mov word [edi],0
    ret

AppendHexByteW:
    push eax
    push ebx
    mov bl,al
    shr al,4
    and eax,0Fh
    mov ax,[HEX_DIGITS_W+eax*2]
    call AppendCharW
    mov al,bl
    and eax,0Fh
    mov ax,[HEX_DIGITS_W+eax*2]
    call AppendCharW
    mov ax,' '
    call AppendCharW
    pop ebx
    pop eax
    ret

AppendHexDword8:
    push ebx
    push ecx
    push edx
    mov ebx,eax
    mov edx,28
.hex_dw_loop:
    mov eax,ebx
    mov cl,dl
    shr eax,cl
    and eax,0Fh
    mov ax,[HEX_DIGITS_W+eax*2]
    call AppendCharW
    test edx,edx
    jz .done
    sub edx,4
    jmp .hex_dw_loop
.done:
    pop edx
    pop ecx
    pop ebx
    ret

ClearHashes:
    mov word [md5Hex],0
    mov word [shaHex],0
    ret

ComputeFileHashes:
    call ClearHashes
    cmp word [filePath],0
    je .ret
    invoke CryptAcquireContext,hProv,0,0,PROV_RSA_AES,CRYPT_VERIFYCONTEXT
    test eax,eax
    jz .ret
    invoke CryptCreateHash,[hProv],CALG_MD5,0,0,hHashMd5
    test eax,eax
    jz .release
    invoke CryptCreateHash,[hProv],CALG_SHA_256,0,0,hHashSha
    test eax,eax
    jz .destroy_md5
    invoke CreateFile,filePath,GENERIC_READ,FILE_SHARE_READ or FILE_SHARE_WRITE,0,OPEN_EXISTING,FILE_ATTRIBUTE_NORMAL,0
    cmp eax,INVALID_HANDLE_VALUE
    je .destroy_both
    mov [hHashFile],eax
.read_loop:
    invoke ReadFile,[hHashFile],hashBuf,HASH_CHUNK,hashRead,0
    test eax,eax
    jz .close_file
    cmp [hashRead],0
    je .close_file
    invoke CryptHashData,[hHashMd5],hashBuf,[hashRead],0
    invoke CryptHashData,[hHashSha],hashBuf,[hashRead],0
    jmp .read_loop
.close_file:
    invoke CloseHandle,[hHashFile]
    mov [hHashFile],0
    mov [hashLen],16
    invoke CryptGetHashParam,[hHashMd5],HP_HASHVAL,md5Bin,hashLen,0
    mov esi,md5Bin
    mov ecx,16
    mov edi,md5Hex
    call BytesToHexW
    mov [hashLen],32
    invoke CryptGetHashParam,[hHashSha],HP_HASHVAL,shaBin,hashLen,0
    mov esi,shaBin
    mov ecx,32
    mov edi,shaHex
    call BytesToHexW
.destroy_both:
    invoke CryptDestroyHash,[hHashSha]
.destroy_md5:
    invoke CryptDestroyHash,[hHashMd5]
.release:
    invoke CryptReleaseContext,[hProv],0
.ret:
    ret

ComputeBufferHashes:
    call ClearHashes
    invoke CryptAcquireContext,hProv,0,0,PROV_RSA_AES,CRYPT_VERIFYCONTEXT
    test eax,eax
    jz .ret
    invoke CryptCreateHash,[hProv],CALG_MD5,0,0,hHashMd5
    test eax,eax
    jz .release
    invoke CryptCreateHash,[hProv],CALG_SHA_256,0,0,hHashSha
    test eax,eax
    jz .destroy_md5

    invoke SendMessage,[hwndEdit],WM_GETTEXTLENGTH,0,0
    mov [tmpLen],eax
    inc eax
    shl eax,1
    invoke GlobalAlloc,GMEM_FIXED or GMEM_ZEROINIT,eax
    test eax,eax
    jz .destroy_both
    mov [textPtr],eax
    mov eax,[tmpLen]
    inc eax
    invoke SendMessage,[hwndEdit],WM_GETTEXT,eax,[textPtr]

    ; Hash current editor contents as UTF-8 with BOM. This is the default
    ; save representation for new files and clearly represents unsaved text.
    mov dword [statusBuf],0BFBBEFh
    invoke CryptHashData,[hHashMd5],statusBuf,3,0
    invoke CryptHashData,[hHashSha],statusBuf,3,0
    invoke WideCharToMultiByte,CP_UTF8,0,[textPtr],[tmpLen],0,0,0,0
    test eax,eax
    jz .free_text
    mov [tmpSize],eax
    invoke GlobalAlloc,GMEM_FIXED or GMEM_ZEROINIT,[tmpSize]
    test eax,eax
    jz .free_text
    mov [convPtr],eax
    invoke WideCharToMultiByte,CP_UTF8,0,[textPtr],[tmpLen],[convPtr],[tmpSize],0,0
    mov [tmpOutLen],eax
    invoke CryptHashData,[hHashMd5],[convPtr],[tmpOutLen],0
    invoke CryptHashData,[hHashSha],[convPtr],[tmpOutLen],0
    invoke GlobalFree,[convPtr]
    mov [convPtr],0
.free_text:
    invoke GlobalFree,[textPtr]
    mov [textPtr],0

    mov [hashLen],16
    invoke CryptGetHashParam,[hHashMd5],HP_HASHVAL,md5Bin,hashLen,0
    mov esi,md5Bin
    mov ecx,16
    mov edi,md5Hex
    call BytesToHexW
    mov [hashLen],32
    invoke CryptGetHashParam,[hHashSha],HP_HASHVAL,shaBin,hashLen,0
    mov esi,shaBin
    mov ecx,32
    mov edi,shaHex
    call BytesToHexW
.destroy_both:
    invoke CryptDestroyHash,[hHashSha]
.destroy_md5:
    invoke CryptDestroyHash,[hHashMd5]
.release:
    invoke CryptReleaseContext,[hProv],0
.ret:
    ret

BytesToHexW:
    test ecx,ecx
    jz .done
.next:
    lodsb
    push ecx
    call AppendHexByteNoSpaceW
    pop ecx
    loop .next
.done:
    mov word [edi],0
    ret

AppendHexByteNoSpaceW:
    push eax
    push ebx
    mov bl,al
    shr al,4
    and eax,0Fh
    mov ax,[HEX_DIGITS_W+eax*2]
    call AppendCharW
    mov al,bl
    and eax,0Fh
    mov ax,[HEX_DIGITS_W+eax*2]
    call AppendCharW
    pop ebx
    pop eax
    ret

AppendFirst8W:
    ; ESI = source wide string, EDI = destination string end
    push ecx
    mov ecx,8
.loop:
    lodsw
    test ax,ax
    jz .done
    stosw
    loop .loop
.done:
    mov word [edi],0
    pop ecx
    ret

DetectLineEndingWide:
    mov [lineEnding],EOL_UNKNOWN
    mov esi,[scanPtr]
.dle_loop:
    mov ax,[esi]
    test ax,ax
    jz .dle_done
    cmp ax,13
    jne .dle_check_lf
    cmp word [esi+2],10
    jne .dle_cr
    mov [lineEnding],EOL_CRLF
    ret
.dle_cr:
    mov [lineEnding],EOL_CR
    ret
.dle_check_lf:
    cmp ax,10
    jne .dle_next
    mov [lineEnding],EOL_LF
    ret
.dle_next:
    add esi,2
    jmp .dle_loop
.dle_done:
    ret

SelectStatusLabels:
    mov eax,[encoding]
    cmp eax,ENC_UTF16LE
    je .ssl_16le
    cmp eax,ENC_UTF16BE
    je .ssl_16be
    mov [encLabel],ENC_UTF8_W
    jmp .ssl_eol
.ssl_16le:
    mov [encLabel],ENC_UTF16LE_W
    jmp .ssl_eol
.ssl_16be:
    mov [encLabel],ENC_UTF16BE_W
.ssl_eol:
    mov eax,[lineEnding]
    cmp eax,EOL_CRLF
    je .ssl_crlf
    cmp eax,EOL_LF
    je .ssl_lf
    cmp eax,EOL_CR
    je .ssl_cr
    mov [eolLabel],EOL_UNKNOWN_W
    ret
.ssl_crlf:
    mov [eolLabel],EOL_CRLF_W
    ret
.ssl_lf:
    mov [eolLabel],EOL_LF_W
    ret
.ssl_cr:
    mov [eolLabel],EOL_CR_W
    ret

ApplyTheme:
    cmp [darkmode],0
    je .light
    invoke CheckMenuItem,[hMenuMain],IDM_DARK,MF_BYCOMMAND or MF_CHECKED
    invoke SendMessage,[hwndEdit],EM_SETBKGNDCOLOR,0,202020h
    cmp [hwndHex],0
    je .dark_no_hex_bg
    invoke SendMessage,[hwndHex],EM_SETBKGNDCOLOR,0,202020h
.dark_no_hex_bg:
    mov [textColor],0E8E8E8h
    call ApplyRichEditTextColor
    invoke InvalidateRect,[hwndEdit],0,TRUE
    invoke InvalidateRect,[hwndStatus],0,TRUE
    invoke InvalidateRect,[hwndStatusInfo],0,TRUE
    invoke InvalidateRect,[hwndStatusVer],0,TRUE
    ret
.light:
    invoke CheckMenuItem,[hMenuMain],IDM_DARK,MF_BYCOMMAND or MF_UNCHECKED
    invoke SendMessage,[hwndEdit],EM_SETBKGNDCOLOR,1,0
    cmp [hwndHex],0
    je .light_no_hex_bg
    invoke SendMessage,[hwndHex],EM_SETBKGNDCOLOR,1,0
.light_no_hex_bg:
    mov [textColor],000000h
    call ApplyRichEditTextColor
    invoke InvalidateRect,[hwndEdit],0,TRUE
    invoke InvalidateRect,[hwndStatus],0,TRUE
    invoke InvalidateRect,[hwndStatusInfo],0,TRUE
    invoke InvalidateRect,[hwndStatusVer],0,TRUE
    ret

ApplyRichEditTextColor:
    ; RichEdit does not use WM_CTLCOLOREDIT for its text color.
    ; Set both the default char format for future typing and all current text.
    invoke RtlZeroMemory,charFmt,CHARFORMATW_SIZE
    mov dword [charFmt],CHARFORMATW_SIZE
    mov dword [charFmt+4],CFM_COLOR
    mov dword [charFmt+8],0
    mov eax,[textColor]
    mov dword [charFmt+20],eax
    invoke SendMessage,[hwndEdit],EM_SETCHARFORMAT,SCF_DEFAULT,charFmt
    invoke SendMessage,[hwndEdit],EM_SETCHARFORMAT,SCF_ALL,charFmt
    cmp [hwndHex],0
    je .ret
    invoke SendMessage,[hwndHex],EM_SETCHARFORMAT,SCF_DEFAULT,charFmt
    invoke SendMessage,[hwndHex],EM_SETCHARFORMAT,SCF_ALL,charFmt
.ret:
    ret

UpdateTitle:
    mov edi,titleBuf
    mov esi,APP_TITLE
    call StrCopyW
    mov esi,TITLE_SEP_W
    call StrCatW
    cmp [modified],0
    je .name
    mov esi,STAR_SPACE_W
    call StrCatW
.name:
    cmp word [filePath],0
    jne .path
    mov esi,UNTITLED_W
    call StrCatW
    jmp .set
.path:
    mov esi,filePath
    call StrCatW
.set:
    invoke SetWindowText,[hwndMain],titleBuf
    ret

UpdateStatus:
    call SelectStatusLabels
    invoke SendMessage,[hwndEdit],EM_GETSEL,selStart,selEnd
    invoke SendMessage,[hwndEdit],EM_LINEFROMCHAR,[selStart],0
    mov ebx,eax
    invoke SendMessage,[hwndEdit],EM_LINEINDEX,ebx,0
    mov ecx,[selStart]
    sub ecx,eax
    inc ebx
    inc ecx
    call FormatStatusLine
    invoke SetWindowText,[hwndStatus],statusBuf
    call FormatInfoLine
    invoke SetWindowText,[hwndStatusInfo],statusBuf
    invoke SetWindowText,[hwndStatusVer],VERSION_W
    ret


; -----------------------------
; ByteForge string/format helpers
; -----------------------------
; Wide-char helpers use EDI as destination and ESI as source.
; They intentionally avoid lstrcpy/lstrcat/lstrlen and wsprintf.

StrCopyW:
.copy_loop:
    lodsw
    stosw
    test ax,ax
    jnz .copy_loop
    sub edi,2
    ret

StrCatW:
.find_end:
    cmp word [edi],0
    je .append
    add edi,2
    jmp .find_end
.append:
    call StrCopyW
    ret

StrLenW:
    xor eax,eax
.len_loop:
    cmp word [esi+eax*2],0
    je .done
    inc eax
    jmp .len_loop
.done:
    ret

AppendUIntW:
    ; input: EAX = unsigned integer, EDI = destination wide string
    ; output: EDI points at terminating NUL after appended number
    push ebx
    push ecx
    push edx
    push esi
    mov esi,numBuf
    add esi,30
    mov word [esi],0
    mov ebx,10
    test eax,eax
    jnz .digits
    sub esi,2
    mov word [esi],'0'
    jmp .append_digits
.digits:
    xor edx,edx
    div ebx
    add dx,'0'
    sub esi,2
    mov [esi],dx
    test eax,eax
    jnz .digits
.append_digits:
    call StrCatW
    pop esi
    pop edx
    pop ecx
    pop ebx
    ret

FormatStatusLine:
    ; EBX = one-based line, ECX = one-based column
    push eax
    mov edi,statusBuf
    mov esi,lnTxt
    call StrCopyW
    mov eax,ebx
    call AppendUIntW
    mov esi,colTxt
    call StrCatW
    mov eax,ecx
    call AppendUIntW
    pop eax
    ret

FormatInfoLine:
    mov edi,statusBuf
    mov esi,[encLabel]
    call StrCopyW
    mov esi,infoSepTxt
    call StrCatW
    mov esi,[eolLabel]
    call StrCatW
    ret

CheckForUpdates:
    ; Manual, privacy-friendly update check. No startup ping, no telemetry:
    ; this only downloads the public version.json when the user asks.
    call ClearUpdateState
    invoke InternetOpen,updateAgentTxt,INTERNET_OPEN_TYPE_PRECONFIG,0,0,0
    test eax,eax
    jz .fail
    mov [hInternet],eax
    invoke InternetOpenUrl,[hInternet],updateUrlTxt,0,0,INTERNET_FLAG_RELOAD or INTERNET_FLAG_NO_CACHE_WRITE,0
    test eax,eax
    jz .close_inet_fail
    mov [hUpdateUrl],eax
    invoke InternetReadFile,[hUpdateUrl],updateBuf,UPDATE_BUF_SIZE-1,updateRead
    test eax,eax
    jz .close_all_fail
    mov eax,[updateRead]
    cmp eax,UPDATE_BUF_SIZE-1
    jbe .term_ready
    mov eax,UPDATE_BUF_SIZE-1
.term_ready:
    mov byte [updateBuf+eax],0
    invoke InternetCloseHandle,[hUpdateUrl]
    mov [hUpdateUrl],0
    invoke InternetCloseHandle,[hInternet]
    mov [hInternet],0
    call ParseUpdateVersion
    test eax,eax
    jz .fail
    call CompareRemoteVersion
    test eax,eax
    jnz .available
    mov edi,updateMsgBuf
    mov esi,updateCurrentLatestTxt
    call StrCopyW
    mov esi,updateCurrentTxt
    call StrCatW
    mov esi,updateLatestTxt
    call StrCatW
    mov esi,remoteVersionText
    call StrCatW
    invoke MessageBox,[hwndMain],updateMsgBuf,updateTitleTxt,MB_OK or MB_ICONINFORMATION
    ret
.available:
    mov edi,updateMsgBuf
    mov esi,updateAvailableTxt
    call StrCopyW
    mov esi,updateCurrentTxt
    call StrCatW
    mov esi,updateLatestTxt
    call StrCatW
    mov esi,remoteVersionText
    call StrCatW
    mov esi,updateDownloadTxt
    call StrCatW
    invoke MessageBox,[hwndMain],updateMsgBuf,updateTitleTxt,MB_OK or MB_ICONINFORMATION
    ret
.close_all_fail:
    cmp [hUpdateUrl],0
    je .close_inet_fail
    invoke InternetCloseHandle,[hUpdateUrl]
    mov [hUpdateUrl],0
.close_inet_fail:
    cmp [hInternet],0
    je .fail
    invoke InternetCloseHandle,[hInternet]
    mov [hInternet],0
.fail:
    invoke MessageBox,[hwndMain],updateFailTxt,updateTitleTxt,MB_OK or MB_ICONWARNING
    ret

ClearUpdateState:
    mov [hInternet],0
    mov [hUpdateUrl],0
    mov [updateRead],0
    mov [remoteVerMajor],0
    mov [remoteVerMinor],0
    mov [remoteVerPatch],0
    mov [remoteVerBuild],0
    mov word [remoteVersionText],0
    mov byte [updateBuf],0
    ret

ParseUpdateVersion:
    mov esi,updateBuf
.search:
    mov al,[esi]
    test al,al
    jz .fail
    cmp al,'"'
    jne .next
    cmp byte [esi+1],'v'
    jne .next
    cmp byte [esi+2],'e'
    jne .next
    cmp byte [esi+3],'r'
    jne .next
    cmp byte [esi+4],'s'
    jne .next
    cmp byte [esi+5],'i'
    jne .next
    cmp byte [esi+6],'o'
    jne .next
    cmp byte [esi+7],'n'
    jne .next
    cmp byte [esi+8],'"'
    jne .next
    add esi,9
    jmp .find_colon
.next:
    inc esi
    jmp .search
.find_colon:
    mov al,[esi]
    test al,al
    jz .fail
    cmp al,':'
    je .find_quote
    inc esi
    jmp .find_colon
.find_quote:
    inc esi
    mov al,[esi]
    test al,al
    jz .fail
    cmp al,'"'
    jne .find_quote
    inc esi
    mov edi,remoteVersionText
    mov [tmpOutLen],0
    mov [tmpLen],0
    mov [tmpSize],0
.version_loop:
    mov al,[esi]
    test al,al
    jz .fail
    cmp al,'"'
    je .done_value
    cmp [tmpSize],31
    jae .skip_copy
    xor ah,ah
    stosw
    inc [tmpSize]
.skip_copy:
    cmp al,'.'
    je .dot
    cmp al,'0'
    jb .fail
    cmp al,'9'
    ja .fail
    mov eax,[tmpLen]
    imul eax,10
    movzx ebx,byte [esi]
    sub ebx,'0'
    add eax,ebx
    mov [tmpLen],eax
    inc esi
    jmp .version_loop
.dot:
    call StoreRemoteVersionComponent
    mov [tmpLen],0
    inc esi
    jmp .version_loop
.done_value:
    call StoreRemoteVersionComponent
    mov word [edi],0
    cmp [tmpSize],0
    je .fail
    mov eax,1
    ret
.fail:
    xor eax,eax
    ret

StoreRemoteVersionComponent:
    mov eax,[tmpOutLen]
    cmp eax,0
    je .major
    cmp eax,1
    je .minor
    cmp eax,2
    je .patch
    cmp eax,3
    je .build
    ret
.major:
    mov eax,[tmpLen]
    mov [remoteVerMajor],eax
    inc [tmpOutLen]
    ret
.minor:
    mov eax,[tmpLen]
    mov [remoteVerMinor],eax
    inc [tmpOutLen]
    ret
.patch:
    mov eax,[tmpLen]
    mov [remoteVerPatch],eax
    inc [tmpOutLen]
    ret
.build:
    mov eax,[tmpLen]
    mov [remoteVerBuild],eax
    inc [tmpOutLen]
    ret

CompareRemoteVersion:
    ; EAX = 1 when remote version is newer than local, otherwise 0.
    mov eax,[remoteVerMajor]
    cmp eax,LOCAL_VER_MAJOR
    ja .newer
    jb .not_newer
    mov eax,[remoteVerMinor]
    cmp eax,LOCAL_VER_MINOR
    ja .newer
    jb .not_newer
    mov eax,[remoteVerPatch]
    cmp eax,LOCAL_VER_PATCH
    ja .newer
    jb .not_newer
    mov eax,[remoteVerBuild]
    cmp eax,LOCAL_VER_BUILD
    ja .newer
.not_newer:
    xor eax,eax
    ret
.newer:
    mov eax,1
    ret

ShowFileInfo:
    call BuildFileInfoText
    cmp [hwndFileInfo],0
    jne .activate
    invoke CreateWindowEx,WS_EX_DLGMODALFRAME,FILEINFO_CLASS,fileInfoTitleTxt,WS_OVERLAPPED or WS_CAPTION or WS_SYSMENU or WS_VISIBLE,CW_USEDEFAULT,CW_USEDEFAULT,860,470,[hwndMain],0,[hInst],0
    mov [hwndFileInfo],eax
    ret
.activate:
    call RefreshFileInfoWindow
    invoke SetActiveWindow,[hwndFileInfo]
    ret

RefreshFileInfoWindow:
    cmp [hwndFileInfoText],0
    je .ret
    call BuildFileInfoText
    invoke SetWindowText,[hwndFileInfoText],fileInfoBuf
    call ApplyFileInfoStatusColor
.ret:
    ret

BuildFileInfoText:
    ; File Info shows the same current-file hash source as the checksum dialog:
    ; disk bytes for an unchanged saved file, otherwise the current editor buffer.
    call SelectStatusLabels
    cmp word [filePath],0
    je .hash_buffer
    cmp [modified],0
    jne .hash_buffer
    call ComputeFileHashes
    jmp .start
.hash_buffer:
    call ComputeBufferHashes
.start:
    mov [fileInfoStatusStart],0
    mov [fileInfoStatusEnd],0
    mov [fileInfoStatusColor],000000h
    mov edi,fileInfoBuf
    mov esi,fileInfoPathTxt
    call StrCopyW
    cmp word [filePath],0
    je .untitled
    mov esi,filePath
    call StrCatW
    jmp .format
.untitled:
    mov esi,UNTITLED_W
    call StrCatW
.format:
    mov esi,fileInfoFormatTxt
    call StrCatW
    mov esi,[encLabel]
    call StrCatW
    mov esi,fileInfoFormatSepTxt
    call StrCatW
    mov esi,[eolLabel]
    call StrCatW
    mov esi,fileInfoBomTxt
    call StrCatW
    mov eax,[fileBomBytes]
    call AppendUIntW
    mov esi,fileInfoLinesTxt
    call StrCatW
    invoke SendMessage,[hwndEdit],EM_GETLINECOUNT,0,0
    call AppendUIntW
    mov esi,fileInfoCharsTxt
    call StrCatW
    invoke SendMessage,[hwndEdit],WM_GETTEXTLENGTH,0,0
    call AppendUIntW
    mov esi,fileInfoSizeTxt
    call StrCatW
    cmp word [filePath],0
    je .size_dash
    invoke GetFileAttributesEx,filePath,0,fileInfoAttr
    test eax,eax
    jz .size_dash
    mov eax,dword [fileInfoAttr+32]
    call AppendUIntW
    mov esi,fileInfoBytesTxt
    call StrCatW
    jmp .modified
.size_dash:
    mov esi,fileInfoDashTxt
    call StrCatW
.modified:
    mov esi,fileInfoModifiedTxt
    call StrCatW
    cmp [modified],0
    je .mod_no
    mov esi,fileInfoYesTxt
    call StrCatW
    jmp .saved
.mod_no:
    mov esi,fileInfoNoTxt
    call StrCatW
.saved:
    mov esi,fileInfoSavedTxt
    call StrCatW
    mov esi,fileInfoBuf
    call StrLenW
    mov [fileInfoStatusStart],eax
    cmp word [filePath],0
    je .never_saved
    invoke GetFileAttributesEx,filePath,0,fileInfoAttr
    test eax,eax
    jz .saved_unavailable
    invoke FileTimeToLocalFileTime,fileInfoAttr+20,localFileTime
    invoke FileTimeToSystemTime,localFileTime,fileSysTime
    movzx eax,word [fileSysTime]
    call AppendUIntW
    mov ax,'-'
    call AppendCharW
    movzx eax,word [fileSysTime+2]
    call AppendUIntW
    mov ax,'-'
    call AppendCharW
    movzx eax,word [fileSysTime+6]
    call AppendUIntW
    mov ax,' '
    call AppendCharW
    movzx eax,word [fileSysTime+8]
    call AppendUIntW
    mov ax,':'
    call AppendCharW
    movzx eax,word [fileSysTime+10]
    call AppendUIntW
    mov ax,':'
    call AppendCharW
    movzx eax,word [fileSysTime+12]
    call AppendUIntW
    cmp [modified],0
    je .saved_green
    mov esi,fileInfoUnsavedTxt
    call StrCatW
    mov [fileInfoStatusColor],0FF0000h
    jmp .saved_end
.saved_green:
    mov [fileInfoStatusColor],008000h
    jmp .saved_end
.saved_unavailable:
    mov esi,fileInfoUnavailableTxt
    call StrCatW
    mov [fileInfoStatusColor],0000FFh
    jmp .saved_end
.never_saved:
    mov esi,fileInfoNeverSavedTxt
    call StrCatW
    mov [fileInfoStatusColor],0000FFh
.saved_end:
    mov esi,fileInfoBuf
    call StrLenW
    mov [fileInfoStatusEnd],eax
    mov esi,fileInfoChecksumSourceTxt
    call StrCatW
    cmp word [filePath],0
    je .source_buffer
    cmp [modified],0
    jne .source_buffer
    mov esi,HEX_SOURCE_FILE_W
    call StrCatW
    jmp .md5
.source_buffer:
    mov esi,HEX_SOURCE_BUFFER_W
    call StrCatW
.md5:
    mov esi,fileInfoMd5Txt
    call StrCatW
    cmp word [md5Hex],0
    je .md5_dash
    mov esi,md5Hex
    call StrCatW
    jmp .sha
.md5_dash:
    mov esi,fileInfoDashTxt
    call StrCatW
.sha:
    mov esi,fileInfoShaTxt
    call StrCatW
    cmp word [shaHex],0
    je .sha_dash
    mov esi,shaHex
    call StrCatW
    jmp .note
.sha_dash:
    mov esi,fileInfoDashTxt
    call StrCatW
.note:
    mov esi,fileInfoNoteTxt
    call StrCatW
    ret

ApplyFileInfoStatusColor:
    cmp [hwndFileInfoText],0
    je .ret
    ; RichEdit normalizes CR/LF internally, so do not reuse string-buffer
    ; offsets here. Color only the value on the "Last saved:" line.
    invoke SendMessage,[hwndFileInfoText],EM_LINEINDEX,6,0
    cmp eax,-1
    je .ret
    add eax,12
    mov [selStart],eax
    invoke SendMessage,[hwndFileInfoText],EM_LINEINDEX,6,0
    mov [tmpPtr],eax
    invoke SendMessage,[hwndFileInfoText],EM_LINELENGTH,eax,0
    add eax,[tmpPtr]
    mov [selEnd],eax
    cmp eax,[selStart]
    jbe .ret
    invoke SendMessage,[hwndFileInfoText],EM_SETSEL,[selStart],[selEnd]
    invoke RtlZeroMemory,charFmt,CHARFORMATW_SIZE
    mov dword [charFmt],CHARFORMATW_SIZE
    mov dword [charFmt+4],CFM_COLOR or CFM_BOLD
    mov dword [charFmt+8],CFE_BOLD
    mov eax,[fileInfoStatusColor]
    mov dword [charFmt+20],eax
    invoke SendMessage,[hwndFileInfoText],EM_SETCHARFORMAT,SCF_SELECTION,charFmt
    invoke SendMessage,[hwndFileInfoText],EM_SETSEL,0,0
.ret:
    ret

ShowFileChecksums:
    ; Existing unchanged file: hash exact bytes on disk.
    ; New/unsaved or modified document: hash current editor buffer.
    cmp word [filePath],0
    je .buffer_hash
    cmp [modified],0
    jne .buffer_hash
.file_hash:
    call ComputeFileHashes
    cmp word [md5Hex],0
    jne .show_file
    invoke MessageBox,[hwndMain],checksumNoHashTxt,checksumTitleTxt,MB_OK or MB_ICONWARNING
    ret
.show_file:
    call ShowChecksumWindow
    ret
.buffer_hash:
    call ComputeBufferHashes
    cmp word [md5Hex],0
    jne .show_buffer
    invoke MessageBox,[hwndMain],checksumNoHashTxt,checksumTitleTxt,MB_OK or MB_ICONWARNING
    ret
.show_buffer:
    call ShowChecksumWindow
    ret

ShowChecksumWindow:
    cmp [hwndChecksum],0
    jne .activate
    invoke CreateWindowEx,WS_EX_DLGMODALFRAME,CHECKSUM_CLASS,checksumTitleTxt,WS_OVERLAPPED or WS_CAPTION or WS_SYSMENU or WS_VISIBLE,CW_USEDEFAULT,CW_USEDEFAULT,600,285,[hwndMain],0,[hInst],0
    mov [hwndChecksum],eax
    ret
.activate:
    invoke SetWindowText,[hwndMd5Actual],md5Hex
    invoke SetWindowText,[hwndShaActual],shaHex
    invoke SetWindowText,[hwndChecksumResult],checksumResultReadyTxt
    invoke SetActiveWindow,[hwndChecksum]
    ret

CompareExpectedChecksums:
    invoke GetWindowText,[hwndMd5Expected],expectedMd5,80
    invoke GetWindowText,[hwndShaExpected],expectedSha,96
    mov [tmpAttr],0
    mov [tmpSize],0
    mov edi,checksumResultBuf
    cmp word [expectedMd5],0
    jne .md5_check
    mov esi,checksumMd5SkippedTxt
    call StrCopyW
    jmp .sha_part
.md5_check:
    push edi
    mov esi,expectedMd5
    mov edi,md5Hex
    call CompareChecksumW
    pop edi
    test eax,eax
    jz .md5_bad
    mov [tmpAttr],1
    mov esi,checksumMd5MatchTxt
    call StrCopyW
    jmp .sha_part
.md5_bad:
    mov [tmpAttr],2
    mov esi,checksumMd5MismatchTxt
    call StrCopyW
.sha_part:
    cmp word [expectedSha],0
    jne .sha_check
    mov esi,checksumShaSkippedTxt
    call StrCatW
    jmp .show
.sha_check:
    push edi
    mov esi,expectedSha
    mov edi,shaHex
    call CompareChecksumW
    pop edi
    test eax,eax
    jz .sha_bad
    mov [tmpSize],1
    mov esi,checksumShaMatchTxt
    call StrCatW
    jmp .show
.sha_bad:
    mov [tmpSize],2
    mov esi,checksumShaMismatchTxt
    call StrCatW
.show:
    invoke SetWindowText,[hwndChecksumResult],checksumResultBuf
    call ApplyChecksumResultFormat
    ret

ApplyChecksumResultFormat:
    invoke SendMessage,[hwndChecksumResult],EM_SETSEL,0,-1
    invoke RtlZeroMemory,charFmt,CHARFORMATW_SIZE
    mov dword [charFmt],CHARFORMATW_SIZE
    mov dword [charFmt+4],CFM_COLOR or CFM_BOLD
    mov dword [charFmt+8],0
    mov dword [charFmt+20],000000h
    invoke SendMessage,[hwndChecksumResult],EM_SETCHARFORMAT,SCF_SELECTION,charFmt
    cmp [tmpAttr],0
    je .sha
    invoke SendMessage,[hwndChecksumResult],EM_LINEINDEX,0,0
    add eax,5
    mov [selStart],eax
    cmp [tmpAttr],1
    jne .md5_bad
    mov [selEnd],eax
    add [selEnd],5
    mov [textColor],008000h
    jmp .md5_color
.md5_bad:
    mov [selEnd],eax
    add [selEnd],8
    mov [textColor],0000FFh
.md5_color:
    call ApplyChecksumWordColor
.sha:
    cmp [tmpSize],0
    je .done
    invoke SendMessage,[hwndChecksumResult],EM_LINEINDEX,1,0
    add eax,9
    mov [selStart],eax
    cmp [tmpSize],1
    jne .sha_bad
    mov [selEnd],eax
    add [selEnd],5
    mov [textColor],008000h
    jmp .sha_color
.sha_bad:
    mov [selEnd],eax
    add [selEnd],8
    mov [textColor],0000FFh
.sha_color:
    call ApplyChecksumWordColor
.done:
    invoke SendMessage,[hwndChecksumResult],EM_SETSEL,0,0
    ret

ApplyChecksumWordColor:
    invoke SendMessage,[hwndChecksumResult],EM_SETSEL,[selStart],[selEnd]
    invoke RtlZeroMemory,charFmt,CHARFORMATW_SIZE
    mov dword [charFmt],CHARFORMATW_SIZE
    mov dword [charFmt+4],CFM_COLOR or CFM_BOLD
    mov dword [charFmt+8],CFE_BOLD
    mov eax,[textColor]
    mov dword [charFmt+20],eax
    invoke SendMessage,[hwndChecksumResult],EM_SETCHARFORMAT,SCF_SELECTION,charFmt
    ret

CompareChecksumW:
    ; ESI = expected, EDI = actual. Case-insensitive exact comparison.
.loop:
    mov ax,[edi]
    test ax,ax
    jz .actual_end
    mov bx,[esi]
    test bx,bx
    jz .no
    call LowerHexPair
    cmp ax,bx
    jne .no
    add esi,2
    add edi,2
    jmp .loop
.actual_end:
    cmp word [esi],0
    jne .no
    mov eax,1
    ret
.no:
    xor eax,eax
    ret

LowerHexPair:
    cmp ax,'A'
    jb .exp
    cmp ax,'Z'
    ja .exp
    add ax,32
.exp:
    cmp bx,'A'
    jb .done
    cmp bx,'Z'
    ja .done
    add bx,32
.done:
    ret

FormatSaveError:
    ; EAX = GetLastError value
    mov ebx,eax
    mov edi,statusBuf
    mov esi,SAVE_FAIL_PREFIX
    call StrCopyW
    mov eax,ebx
    call AppendUIntW
    ret

section '.rsrc' resource data readable

  directory RT_ICON,icons,\
            RT_GROUP_ICON,group_icons,\
            RT_VERSION,versions

  resource icons,\
           1,0409h,app_icon_16,\
           2,0409h,app_icon_32

  resource group_icons,\
           IDI_APP,0409h,app_icon_group

  resource versions,\
           1,0409h,version

  ; The complete ByteForge icon lives in this source file.  The two 32-bit
  ; DIB payloads give Windows native title-bar and Explorer sizes without a
  ; sidecar .ico file or a runtime dependency.
  macro byteforge_icon label,icon_width,icon_height
   { local data
     label dd RVA data,40+icon_width*icon_height*4+icon_height*4,0,0
     data:
       dd 40,icon_width,icon_height*2
       dw 1,32
       dd 0,icon_width*icon_height*4,0,0,0,0

     bf_row = 0
     while bf_row < icon_height
       bf_col = 0
       while bf_col < icon_width
         bf_x = bf_col*32/icon_width
         bf_y = (icon_height-1-bf_row)*32/icon_height
         bf_pixel = 0FF1B1E24h       ; forged charcoal background

         ; Light B monogram.
         if bf_x >= 6
           if bf_x <= 8
             if bf_y >= 7
               if bf_y <= 24
                 bf_pixel = 0FFF1F4F6h
               end if
             end if
           end if
         end if
         if bf_x >= 9
           if bf_x <= 13
             if (bf_y >= 7) & (bf_y <= 9)
               bf_pixel = 0FFF1F4F6h
             end if
             if (bf_y >= 14) & (bf_y <= 16)
               bf_pixel = 0FFF1F4F6h
             end if
             if (bf_y >= 22) & (bf_y <= 24)
               bf_pixel = 0FFF1F4F6h
             end if
           end if
         end if
         if bf_x >= 13
           if bf_x <= 15
             if (bf_y >= 9) & (bf_y <= 13)
               bf_pixel = 0FFF1F4F6h
             end if
             if (bf_y >= 17) & (bf_y <= 21)
               bf_pixel = 0FFF1F4F6h
             end if
           end if
         end if

         ; Forge-orange F monogram and sparks.
         if bf_x >= 18
           if bf_x <= 20
             if bf_y >= 7
               if bf_y <= 24
                 bf_pixel = 0FFFF982Eh
               end if
             end if
           end if
         end if
         if bf_x >= 21
           if bf_x <= 27
             if (bf_y >= 7) & (bf_y <= 9)
               bf_pixel = 0FFFF982Eh
             end if
           end if
         end if
         if bf_x >= 21
           if bf_x <= 25
             if (bf_y >= 14) & (bf_y <= 16)
               bf_pixel = 0FFFF982Eh
             end if
           end if
         end if
         if (bf_x >= 24) & (bf_x <= 25) & (bf_y >= 3) & (bf_y <= 4)
           bf_pixel = 0FFFF982Eh
         end if
         if (bf_x >= 28) & (bf_x <= 29) & (bf_y >= 5) & (bf_y <= 6)
           bf_pixel = 0FFFF982Eh
         end if

         dd bf_pixel
         bf_col = bf_col+1
       end while
       bf_row = bf_row+1
     end while

     ; One DWORD-wide, fully opaque AND-mask row for both icon sizes.
     repeat icon_height
       dd 0
     end repeat
     align 4 }

  byteforge_icon app_icon_16,16,16
  byteforge_icon app_icon_32,32,32

  app_icon_group dd RVA app_icon_group_data,34,0,0
  app_icon_group_data:
    dw 0,1,2
    db 16,16,0,0
    dw 1,32
    dd 40+16*16*4+16*4
    dw 1
    db 32,32,0,0
    dw 1,32
    dd 40+32*32*4+32*4
    dw 2
    align 4

  versioninfo version,4,1,0,0409h,04E4h,\
              'FileDescription','Small and fast text editor without fluff',\
              'FileVersion','1.0.0.10',\
              'InternalName','ByteForge',\
              'OriginalFilename','Byteforge.exe',\
              'ProductName','ByteForge',\
              'ProductVersion','1.0.0.10'

section '.idata' import data readable writeable

library kernel32,'KERNEL32.DLL',\
        user32,'USER32.DLL',\
        gdi32,'GDI32.DLL',\
        comdlg32,'COMDLG32.DLL',\
        shell32,'SHELL32.DLL',\
        advapi32,'ADVAPI32.DLL',\
        wininet,'WININET.DLL'

include 'api\kernel32.inc'
include 'api\user32.inc'
include 'api\gdi32.inc'
include 'api\comdlg32.inc'
import shell32,\
       DragAcceptFiles,'DragAcceptFiles',\
       DragFinish,'DragFinish',\
       DragQueryFile,'DragQueryFileW',\
       CommandLineToArgvW,'CommandLineToArgvW'
import advapi32,\
       CryptAcquireContext,'CryptAcquireContextW',\
       CryptCreateHash,'CryptCreateHash',\
       CryptHashData,'CryptHashData',\
       CryptGetHashParam,'CryptGetHashParam',\
       CryptDestroyHash,'CryptDestroyHash',\
       CryptReleaseContext,'CryptReleaseContext'
import wininet,\
       InternetOpen,'InternetOpenW',\
       InternetOpenUrl,'InternetOpenUrlW',\
       InternetReadFile,'InternetReadFile',\
       InternetCloseHandle,'InternetCloseHandle'
