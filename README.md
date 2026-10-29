# Chuyển đổi hồ sơ SketchUp (SketchUp Profile Switcher)

<p align="center">
  <img src="public/icon.svg" width="96" height="96" alt="SketchUp Profile Switcher Logo" />
</p>

<p align="center">
  <strong>Công cụ quản lý, chuyển đổi đa hồ sơ SketchUp siêu nhẹ, an toàn tuyệt đối và hiệu năng cao.</strong><br>
  Xây dựng trên nền tảng <strong>Tauri v2</strong>, <strong>Svelte 5</strong> và <strong>Rust</strong>.
</p>

<p align="center">
  <a href="#-tính-năng-nổi-bật">Tính năng</a> •
  <a href="#-hình-ảnh-giao-diện">Giao diện</a> •
  <a href="#-tải-về--cài-đặt">Tải về</a> •
  <a href="#-hướng-dẫn-sử-dụng">Sử dụng</a> •
  <a href="#-công-nghệ">Công nghệ</a> •
  <a href="README.en.md">English Documentation</a>
</p>

---

## 📸 Hình ảnh giao diện

| Giao diện chính (Dark Mode) | Xác nhận xóa an toàn |
| :---: | :---: |
| ![Giao diện chính](docs/screenshots/main_interface.png) | ![Xác nhận xóa](docs/screenshots/delete_modal.png) |

---

## ⚡ Tính năng nổi bật

- 🚀 **Siêu nhẹ & Tốc độ tức thì**: Bộ nhớ tiêu thụ cực thấp (~20MB RAM), không phụ thuộc vào .NET Runtime hay Chromium cồng kềnh.
- 🎨 **Tùy biến Hồ sơ trực quan**: Tự do đổi biểu tượng Phosphor Icons (10 bộ icon chuyên dụng kiến trúc/đồ họa), chọn màu sắc nhận diện và gán tên gợi nhớ.
- 👁️ **Xem chi tiết hồ sơ**: Xem nhanh danh sách plugin đã cài đặt và so sánh phím tắt tùy chỉnh (tự động so sánh với phím tắt gốc của SketchUp 2026).
- 💾 **Lưu hồ sơ nhanh từ khay hệ thống**: Hộp thoại lưu nhanh từ System Tray cho phép đặt tên, chọn biểu tượng và màu sắc trực tiếp không cần mở cửa sổ chính.
- ⌨️ **Phím tắt toàn cục (Global Hotkey)**: Gán tổ hợp phím (ví dụ: `Ctrl+Alt+1`, `Ctrl+Shift+D`) để chuyển đổi hồ sơ ngay lập tức khi đang ở ngoài màn hình desktop hay trong phần mềm khác.
- 🔔 **Thông báo Windows Native**: Sử dụng trực tiếp Windows Toast Notification API (WinRT AUMID) chuẩn hệ điều hành với biểu tượng ứng dụng chính thức, âm thanh và hiển thị trong Action Center.
- 🛡️ **Bảo vệ toàn vẹn dữ liệu**: Tự động phát hiện khi SketchUp đang chạy, đưa ra hộp thoại cảnh báo trước khi sao chép và hỗ trợ nút **Buộc đóng SketchUp** một chạm.
- 🔍 **Tự động nhận diện đa phiên bản**: Hỗ trợ toàn diện từ SketchUp 2021 đến SketchUp 2026 (quét cả thư mục tiêu chuẩn lẫn cấu trúc thư mục con của Trimble).
- 🧩 **Bảo vệ Plugin Render độc lập**: Tùy chọn giữ nguyên các plugin render phức tạp (V-Ray, Enscape, D5 Render, Thea Render) tránh lỗi license khi đổi hồ sơ.
- 📦 **Sao lưu & Nhập xuất Zip**: Đóng gói toàn bộ hồ sơ thành file `.zip` để chia sẻ cho đồng nghiệp hoặc chuyển máy trạm mới dễ dàng.

---

## 📥 Tải về & Cài đặt

Truy cập trang [**GitHub Releases**](https://github.com/phathua/su-profile-switcher/releases) để tải về phiên bản phù hợp với máy tính của bạn:

### Gói cài đặt tự động (NSIS Setup - Khuyên dùng)
- `SuPS-x64-v*-Setup.exe`: Bản cài đặt tự động cho Windows 64-bit tiêu chuẩn.
- `SuPS-x86-v*-Setup.exe`: Bản cài đặt tự động cho Windows 32-bit.
- `SuPS-arm64-v*-Setup.exe`: Bản cài đặt tự động cho Windows on ARM (Surface Pro Copilot+, Snapdragon X Elite).

> 💡 **Cài đặt trong AppData**: Trình cài đặt tự động cài vào thư mục `%LOCALAPPDATA%\Programs\SketchUp Profile Switcher` theo quyền người dùng hiện tại, không yêu cầu quyền Quản trị viên (UAC) và hỗ trợ tự động nâng cấp mượt mà.

---

## 🖥️ Hướng dẫn sử dụng

1. **Chọn phiên bản SketchUp**: Chọn phiên bản bạn đang dùng ở góc phải thanh điều hướng.
2. **Tạo hồ sơ mới**: Nhấn vào nút `+ Tạo hồ sơ`, đặt tên và chọn các thành phần muốn lưu (Plugins, Phím tắt, Thanh công cụ, Vật liệu, Mẫu template).
3. **Xem chi tiết hồ sơ**: Bấm nút hình con mắt cạnh hồ sơ để duyệt danh sách plugin đang cài và xem sự khác biệt phím tắt so với bản gốc SketchUp 2026.
4. **Tùy chỉnh biểu tượng & Phím tắt**: Bấm biểu tượng cây bút tại dòng hồ sơ, chọn icon, màu sắc và bấm tổ hợp phím muốn gán.
5. **Chuyển đổi hồ sơ**:
   - Nhấn vào biểu tượng chuyển đổi cạnh hồ sơ trong danh sách.
   - Hoặc click chuột phải vào icon ứng dụng dưới khay hệ thống (System Tray).
   - Hoặc bấm trực tiếp phím tắt toàn cục đã gán.
6. **Lưu nhanh từ khay**: Click chuột phải vào icon ở khay hệ thống và chọn `Lưu hồ sơ hiện tại...` để mở cửa sổ lưu nhanh có sẵn bộ chọn màu và icon.
7. **Chạy & Đóng SketchUp**: Nút ở góc trên sẽ hiển thị `Chạy SketchUp` (màu xanh) khi phần mềm chưa mở, và tự động chuyển thành `Buộc đóng SketchUp` (màu đỏ) khi SketchUp đang chạy.

---

## Giao diện dòng lệnh (CLI - sups)

Ứng dụng đi kèm công cụ dòng lệnh native console `sups` siêu nhẹ (~1.3MB), khởi động dưới 5ms:

```bash
# Xem danh sách hồ sơ (dạng văn bản hoặc JSON)
sups list --version "SketchUp 2026"
sups list --json

# Xem hồ sơ đang kích hoạt và mã băm cấu hình
sups current

# Chuyển đổi hồ sơ an toàn
sups switch "Production V-Ray" --force

# Lưu trạng thái hiện tại thành hồ sơ mới
sups save "Backup_2026" --no-shortcuts

# Kiểm tra tiến trình SketchUp
sups sketchup status --json
```

---

## Máy chủ Model Context Protocol (MCP Server)

Tích hợp trực tiếp MCP Server chuẩn hóa JSON-RPC 2.0 stdio, cho phép các AI Agent (Antigravity, Claude Desktop, Cursor, Codex) tự động kiểm tra và chuyển đổi không gian làm việc SketchUp:

```bash
sups mcp
```

Xem tài liệu chi tiết tại [docs/mcp/README.md](docs/mcp/README.md):
- [Hướng dẫn cho AI Agent](docs/mcp/ai_agent_guide.md)
- [Hướng dẫn tích hợp các phần mềm AI](docs/mcp/integration_guide.md)
- [Bảng đặc tả 8 công cụ MCP](docs/mcp/tools_reference.md)

---

## 🛠️ Công nghệ phát triển

- **Core Engine**: [Rust](https://www.rust-lang.org/) + [Tauri v2](https://v2.tauri.app/) (WinRT Notification, Windows 0.57)
- **Frontend Framework**: [Svelte 5](https://svelte.dev/) (Runes reactivity)
- **Styling**: [Tailwind CSS v4](https://tailwindcss.com/)
- **Icon System**: [Phosphor Icons](https://phosphoricons.com/)
- **Installer**: Gói cài đặt NSIS tự động phân phối theo người dùng (Per-user AppData)
- **CI/CD Pipeline**: GitHub Actions với Node.js 24, tự động đóng gói đa kiến trúc (x64, x86, arm64) kèm nhật ký thay đổi tự động.

---

## 📄 Giấy phép

Phát hành dưới giấy phép [MIT License](LICENSE).
