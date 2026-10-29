# SketchUp Profile Switcher MCP Server

Native, high-performance [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) server for Trimble SketchUp profile management, extension auditing, and workspace orchestration.

Built directly into the **SketchUp Profile Switcher** core using native Rust.

---

## 1. Architectural Highlights

- **Zero Runtime Dependencies**: Written entirely in native Rust. No Node.js, Python, or .NET runtime required on the host system.
- **Microsecond Cold Start**: Stdio process spins up in under 5 milliseconds.
- **Ultra-low Memory Footprint**: Operates within 6–12 MB of RAM, conforming to strict system constraints.
- **Safe State Transitions**: Integrated process inspection detects running SketchUp instances to prevent file corruption.
- **Render Engine Preservation**: Automatically protects complex rendering engines (V-Ray, Enscape, D5 Render, Thea) from licensing breaks during profile switching.

---

## 2. Quick Start for AI Agents & Clients

### Command-Line Stdio Transport

Run the native CLI binary in MCP mode:

```bash
# Using dedicated CLI console binary:
sups mcp

# Or using the primary multi-call executable:
su-profile-switcher.exe mcp
```

The server communicates via standard JSON-RPC 2.0 protocol over `stdin`/`stdout`. All diagnostic logs and errors are isolated to `stderr` to ensure protocol stream integrity.

---

## 3. Documentation Sitemap

- [AI Agent Operating Guide](ai_agent_guide.md): Specialized execution manual, invariant safety rules, and canonical workflows for AI agents (Antigravity, Claude, Codex, Cursor).
- [Integration Guide](integration_guide.md): Copy-paste configuration guides for Antigravity, Claude Desktop, Claude Code, Cursor, Windsurf, and OpenAI Codex tools.
- [Tools Reference](tools_reference.md): Detailed input schemas, parameter requirements, and return types for all 8 MCP tools.
