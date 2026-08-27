# Cooldown — macOS Kurulum Rehberi

Bu rehber, Cooldown uygulamasını **Mac** üzerinde derlemek ve çalıştırmak için gereken adımları anlatır.

## Gereksinimler

| Araç | Minimum sürüm |
|------|----------------|
| macOS | 10.15 (Catalina) veya üzeri |
| Xcode Command Line Tools | `xcode-select --install` |
| Rust | 1.70+ — [rustup.rs](https://rustup.rs) |
| Node.js | 18+ — [nodejs.org](https://nodejs.org) |

## 1. Projeyi hazırlayın

```bash
git clone <repo-url> cooldown
cd cooldown
npm install
npm run icons
```

`npm run icons` PNG, ICO ve ICNS ikon dosyalarını oluşturur (macOS `.app` paketi için `icon.icns` zorunludur).

## 2. Geliştirme modunda çalıştırın

```bash
npm run tauri dev
```

İlk açılışta macOS, Cooldown'un **Erişilebilirlik** ve **Girdi İzleme** izinlerini isteyebilir. Bu izinler olmadan pencere takibi ve klavye temposu ölçümü çalışmaz.

## 3. macOS izinleri (önemli)

**Sistem Ayarları → Gizlilik ve Güvenlik** bölümünden Cooldown'a şu izinleri verin:

| İzin | Neden gerekli? |
|------|----------------|
| **Erişilebilirlik** (Accessibility) | Hangi uygulamada çalıştığınızı algılar |
| **Girdi İzleme** (Input Monitoring) | Yazma temposunu ölçer (içerik okunmaz) |

> Cooldown yalnızca tuş sayısını ve pencere başlıklarını kullanır; yazdığınız metinleri kaydetmez.

## 4. Terminal entegrasyonu (Zsh)

macOS'ta varsayılan kabuk genellikle **Zsh**'dir. `~/.zshrc` dosyanıza ekleyin:

```bash
cooldown_report() {
  local ec=$?
  if [ $ec -ne 0 ]; then
    curl -sf -X POST http://127.0.0.1:9876/event \
      -H 'Content-Type: application/json' \
      -d "{\"source\":\"terminal\",\"exit_code\":$ec}" >/dev/null 2>&1
  fi
}
precmd_functions+=(cooldown_report)
```

Ardından: `source ~/.zshrc`

## 5. VS Code / Cursor eklentisi

```bash
cd integrations/vscode-cooldown
npm install
npm run compile
```

VS Code veya Cursor'da **Extensions → Install from VSIX** ya da geliştirici modunda yükleyin. Eklenti build/lint hatalarını Cooldown'a iletir.

## 6. Production derlemesi (.dmg / .app)

```bash
npm run tauri build
```

Çıktı:

```
src-tauri/target/release/bundle/macos/Cooldown.app
src-tauri/target/release/bundle/dmg/Cooldown_0.2.0_aarch64.dmg   # Apple Silicon
src-tauri/target/release/bundle/dmg/Cooldown_0.2.0_x64.dmg        # Intel
```

`.app` dosyasını **Applications** klasörüne sürükleyip çalıştırın.

### İmzasız uygulama uyarısı

Kendi makinenizde derlediyseniz ve Apple Developer sertifikanız yoksa, macOS ilk açılışta uyarı verebilir:

**Sistem Ayarları → Gizlilik ve Güvenlik → "Yine de Aç"**

veya terminalden:

```bash
xattr -cr /Applications/Cooldown.app
```

## 7. Mac'e özel notlar

- **Menü çubuğu simgesi:** Cooldown, Windows'taki tray gibi Mac menü çubuğunda (sağ üst) çalışır.
- **Ekran kilidi algılama:** Mac kilitlendiğinde mola süresi otomatik sayılır.
- **Screen Time:** Code, Cursor, Terminal, iTerm, Safari, Slack vb. Mac uygulamaları tanınır.
- **PowerShell hook:** Mac'te gerekli değil; Zsh yeterlidir.
- **Veritabanı konumu:** `~/Library/Application Support/com.cooldown.app/cooldown.db`

## Sorun giderme

| Sorun | Çözüm |
|-------|-------|
| Fatigue Score güncellenmiyor | Erişilebilirlik iznini kontrol edin |
| Klavye metrikleri 0 | Girdi İzleme iznini verin, uygulamayı yeniden başlatın |
| Terminal hataları sayılmıyor | Cooldown çalışıyor mu? Zsh hook yüklü mü? |
| `icon.icns` bulunamadı | `npm run icons` çalıştırın |
| Derleme hatası (linker) | `xcode-select --install` |

## Hızlı kontrol listesi

```bash
# Rust
rustc --version

# Node
node --version

# İkonlar
ls src-tauri/icons/icon.icns

# HTTP sunucusu (uygulama açıkken)
curl -s http://127.0.0.1:9876/event -X POST \
  -H 'Content-Type: application/json' \
  -d '{"source":"test","message":"ping"}'
```

---

Sorularınız için proje README'sine veya GitHub Issues'a bakın.
