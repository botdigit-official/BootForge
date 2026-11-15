//! Embedded Web Desktop UI for Botdigit BootForge.

pub const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Botdigit BootForge — Universal Boot & Recovery Studio</title>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@300;400;500;600;700;800&family=JetBrains+Mono:wght@400;500;700&display=swap" rel="stylesheet">
  <style>
    :root {
      --bg: #07090E;
      --bg-surface: #0E131F;
      --bg-card: #141C2E;
      --bg-card-hover: #1B263E;
      --border: #23304B;
      --border-focus: #10B981;
      --text: #F8FAFC;
      --text-muted: #94A3B8;
      --text-dim: #64748B;
      --accent: #10B981;
      --accent-hover: #059669;
      --accent-glow: rgba(16, 185, 129, 0.28);
      --cyan: #06B6D4;
      --cyan-glow: rgba(6, 182, 212, 0.28);
      --blue: #3B82F6;
      --blue-glow: rgba(59, 130, 246, 0.28);
      --amber: #F59E0B;
      --amber-glow: rgba(245, 158, 11, 0.28);
      --purple: #8B5CF6;
      --purple-glow: rgba(139, 92, 246, 0.28);
      --danger: #EF4444;
      --radius: 14px;
      --font: 'Plus Jakarta Sans', -apple-system, BlinkMacSystemFont, sans-serif;
      --font-mono: 'JetBrains Mono', monospace;
    }

    * { box-sizing: border-box; margin: 0; padding: 0; }

    body {
      background: var(--bg);
      color: var(--text);
      font-family: var(--font);
      min-height: 100vh;
      display: flex;
      flex-direction: column;
      -webkit-font-smoothing: antialiased;
      user-select: none;
    }

    header {
      background: rgba(14, 19, 31, 0.9);
      backdrop-filter: blur(20px);
      border-bottom: 1px solid var(--border);
      padding: 12px 28px;
      display: flex;
      align-items: center;
      justify-content: space-between;
      position: sticky;
      top: 0;
      z-index: 50;
    }

    .brand {
      display: flex;
      align-items: center;
      gap: 12px;
    }

    .brand-icon {
      width: 40px;
      height: 40px;
      background: linear-gradient(135deg, var(--accent) 0%, #059669 100%);
      border-radius: 10px;
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 22px;
      box-shadow: 0 0 20px var(--accent-glow);
    }

    .brand-title {
      font-size: 17px;
      font-weight: 800;
      letter-spacing: -0.5px;
      background: linear-gradient(135deg, #F8FAFC 0%, #94A3B8 100%);
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
    }

    .brand-subtitle {
      font-size: 11px;
      color: var(--cyan);
      font-weight: 600;
      letter-spacing: 0.5px;
      text-transform: uppercase;
      font-family: var(--font-mono);
    }

    /* Robotic Core Telemetry Ribbon */
    .telemetry-ribbon {
      display: flex;
      align-items: center;
      gap: 16px;
      background: rgba(7, 9, 14, 0.8);
      border: 1px solid var(--border);
      padding: 6px 14px;
      border-radius: 30px;
      font-family: var(--font-mono);
      font-size: 11px;
    }

    .telemetry-item {
      display: flex;
      align-items: center;
      gap: 6px;
      color: var(--text-muted);
    }

    .status-dot {
      width: 8px;
      height: 8px;
      border-radius: 50%;
      background: var(--accent);
      box-shadow: 0 0 10px var(--accent);
      animation: pulse 2s infinite ease-in-out;
    }

    @keyframes pulse {
      0%, 100% { opacity: 1; transform: scale(1); }
      50% { opacity: 0.5; transform: scale(0.85); }
    }

    /* Main Navigation Tabs */
    .nav-tabs {
      display: flex;
      gap: 6px;
      background: rgba(7, 9, 14, 0.6);
      padding: 4px;
      border-radius: 12px;
      border: 1px solid var(--border);
    }

    .tab-btn {
      background: transparent;
      border: none;
      color: var(--text-muted);
      font-family: var(--font);
      font-size: 13px;
      font-weight: 600;
      padding: 8px 14px;
      border-radius: 8px;
      cursor: pointer;
      display: flex;
      align-items: center;
      gap: 6px;
      transition: all 0.2s ease;
    }

    .tab-btn:hover {
      color: var(--text);
      background: rgba(255, 255, 255, 0.04);
    }

    .tab-btn.active {
      background: var(--bg-card);
      color: #34D399;
      border: 1px solid rgba(16, 185, 129, 0.3);
      box-shadow: 0 2px 10px rgba(0, 0, 0, 0.3);
    }

    /* Main Layout Grid */
    main {
      flex: 1;
      display: grid;
      grid-template-columns: 340px 1fr;
      gap: 20px;
      padding: 24px 28px;
      max-width: 1700px;
      margin: 0 auto;
      width: 100%;
    }

    .panel {
      background: var(--bg-surface);
      border: 1px solid var(--border);
      border-radius: var(--radius);
      padding: 20px;
      display: flex;
      flex-direction: column;
      gap: 16px;
      box-shadow: 0 8px 30px rgba(0, 0, 0, 0.4);
    }

    .panel-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      border-bottom: 1px solid rgba(255, 255, 255, 0.05);
      padding-bottom: 12px;
    }

    .panel-title {
      font-size: 15px;
      font-weight: 700;
      display: flex;
      align-items: center;
      gap: 8px;
    }

    /* Storage Device Cards */
    .device-card {
      background: var(--bg-card);
      border: 1px solid var(--border);
      border-radius: 10px;
      padding: 14px;
      cursor: pointer;
      transition: all 0.2s ease;
      position: relative;
    }

    .device-card:hover {
      border-color: rgba(16, 185, 129, 0.4);
      background: var(--bg-card-hover);
    }

    .device-card.selected {
      border-color: var(--accent);
      background: rgba(16, 185, 129, 0.08);
      box-shadow: 0 0 16px var(--accent-glow);
    }

    .device-card.system-locked {
      opacity: 0.55;
      cursor: not-allowed;
      border-color: rgba(239, 68, 68, 0.3);
    }

    .device-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      margin-bottom: 6px;
    }

    .device-name {
      font-weight: 700;
      font-size: 14px;
    }

    .device-capacity {
      font-family: var(--font-mono);
      font-weight: 700;
      font-size: 13px;
      color: #34D399;
    }

    .device-meta {
      font-size: 12px;
      color: var(--text-dim);
      font-family: var(--font-mono);
      display: flex;
      gap: 8px;
    }

    .badge {
      display: inline-flex;
      align-items: center;
      gap: 4px;
      font-size: 10px;
      font-weight: 700;
      padding: 2px 7px;
      border-radius: 6px;
      text-transform: uppercase;
      font-family: var(--font-mono);
    }

    .badge-emerald { background: rgba(16, 185, 129, 0.15); color: #34D399; border: 1px solid rgba(16, 185, 129, 0.3); }
    .badge-cyan { background: rgba(6, 182, 212, 0.15); color: #67E8F9; border: 1px solid rgba(6, 182, 212, 0.3); }
    .badge-amber { background: rgba(245, 158, 11, 0.15); color: #FCD34D; border: 1px solid rgba(245, 158, 11, 0.3); }
    .badge-red { background: rgba(239, 68, 68, 0.15); color: #FCA5A5; border: 1px solid rgba(239, 68, 68, 0.3); }
    .badge-purple { background: rgba(139, 92, 246, 0.15); color: #C4B5FD; border: 1px solid rgba(139, 92, 246, 0.3); }

    /* Content Area & Tabs */
    .content-area {
      display: flex;
      flex-direction: column;
      gap: 20px;
    }

    .tab-content {
      display: none;
      flex-direction: column;
      gap: 20px;
    }

    .tab-content.active {
      display: flex;
    }

    /* Dropzone & File Pickers */
    .dropzone {
      border: 2px dashed rgba(16, 185, 129, 0.35);
      background: rgba(16, 185, 129, 0.03);
      border-radius: var(--radius);
      padding: 30px 20px;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      gap: 12px;
      cursor: pointer;
      transition: all 0.2s ease;
      text-align: center;
    }

    .dropzone:hover {
      border-color: var(--accent);
      background: rgba(16, 185, 129, 0.08);
      box-shadow: 0 0 24px var(--accent-glow);
    }

    .picker-buttons {
      display: flex;
      gap: 10px;
      flex-wrap: wrap;
      justify-content: center;
    }

    /* Buttons */
    .btn-primary {
      background: linear-gradient(135deg, var(--accent) 0%, #059669 100%);
      color: #07090E;
      font-weight: 800;
      font-size: 14px;
      padding: 12px 22px;
      border-radius: 10px;
      border: none;
      cursor: pointer;
      box-shadow: 0 4px 20px var(--accent-glow);
      transition: all 0.2s ease;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: 8px;
    }

    .btn-primary:hover {
      transform: translateY(-1px);
      box-shadow: 0 6px 28px rgba(16, 185, 129, 0.45);
    }

    .btn-secondary {
      background: var(--bg-card);
      color: var(--text);
      font-weight: 600;
      font-size: 13px;
      padding: 10px 16px;
      border-radius: 8px;
      border: 1px solid var(--border);
      cursor: pointer;
      transition: all 0.2s ease;
      display: inline-flex;
      align-items: center;
      gap: 8px;
    }

    .btn-secondary:hover {
      border-color: var(--cyan);
      background: var(--bg-card-hover);
    }

    .btn-danger {
      background: rgba(239, 68, 68, 0.15);
      color: #FCA5A5;
      border: 1px solid rgba(239, 68, 68, 0.35);
      font-weight: 700;
      font-size: 13px;
      padding: 10px 16px;
      border-radius: 8px;
      cursor: pointer;
      transition: all 0.2s ease;
    }

    .btn-danger:hover {
      background: rgba(239, 68, 68, 0.25);
      border-color: var(--danger);
    }

    /* Cards Grid */
    .cards-grid {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
      gap: 14px;
    }

    .smart-card {
      background: var(--bg-card);
      border: 1px solid var(--border);
      border-radius: 12px;
      padding: 16px;
      display: flex;
      flex-direction: column;
      gap: 12px;
      transition: all 0.2s ease;
      position: relative;
    }

    .smart-card:hover {
      border-color: rgba(6, 182, 212, 0.4);
      transform: translateY(-2px);
      box-shadow: 0 6px 24px rgba(0, 0, 0, 0.35);
    }

    .smart-card-top {
      display: flex;
      align-items: center;
      gap: 12px;
    }

    .card-icon {
      width: 44px;
      height: 44px;
      border-radius: 10px;
      background: rgba(7, 9, 14, 0.8);
      border: 1px solid var(--border);
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 24px;
    }

    /* Telemetry Data Grid */
    .telemetry-grid {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
      gap: 14px;
    }

    .telemetry-card {
      background: var(--bg-card);
      border: 1px solid var(--border);
      border-radius: 10px;
      padding: 16px;
      display: flex;
      flex-direction: column;
      gap: 6px;
    }

    .telemetry-label {
      font-size: 11px;
      color: var(--text-dim);
      font-weight: 700;
      text-transform: uppercase;
      font-family: var(--font-mono);
    }

    .telemetry-value {
      font-size: 16px;
      font-weight: 700;
      color: #F8FAFC;
    }

    /* Terminal & Progress Log */
    .terminal-box {
      background: #04060A;
      border: 1px solid var(--border);
      border-radius: 10px;
      padding: 16px;
      font-family: var(--font-mono);
      font-size: 12px;
      color: #34D399;
      max-height: 220px;
      overflow-y: auto;
      white-space: pre-wrap;
      line-height: 1.5;
    }

    /* Modals */
    .modal-backdrop {
      display: none;
      position: fixed;
      inset: 0;
      background: rgba(0, 0, 0, 0.85);
      backdrop-filter: blur(14px);
      z-index: 100;
      align-items: center;
      justify-content: center;
      padding: 20px;
    }

    .modal-box {
      background: var(--bg-surface);
      border: 1px solid var(--border);
      border-radius: 16px;
      max-width: 680px;
      width: 100%;
      max-height: 88vh;
      overflow-y: auto;
      padding: 28px;
      display: flex;
      flex-direction: column;
      gap: 20px;
      box-shadow: 0 24px 60px rgba(0, 0, 0, 0.8);
    }

    /* Progress bar */
    .progress-track {
      background: #0E131F;
      border: 1px solid var(--border);
      border-radius: 20px;
      height: 12px;
      overflow: hidden;
      position: relative;
    }

    .progress-bar-fill {
      background: linear-gradient(90deg, #10B981, #06B6D4);
      height: 100%;
      width: 0%;
      transition: width 0.2s ease;
      box-shadow: 0 0 14px var(--accent);
    }
    /* ── Simple 3-step flow ── */
    .step-num { display:inline-flex; align-items:center; justify-content:center; width:24px; height:24px; border-radius:50%;
      background: var(--accent, #10B981); color:#04130D; font-weight:800; font-size:13px; margin-right:8px; flex:none; }
    .tool-row { display:flex; gap:12px; align-items:flex-start; padding:10px 12px; border:1px solid var(--border);
      border-radius:10px; cursor:pointer; background: var(--bg-card); }
    .tool-row:hover { border-color: rgba(16,185,129,0.5); }
    .tool-row input { margin-top:3px; width:16px; height:16px; accent-color:#10B981; flex:none; }
    .tool-name { font-weight:700; font-size:14px; }
    .tool-size { font-family: var(--font-mono); font-size:11px; color: var(--text-dim); margin-left:6px; }
    .tool-purpose { font-size:12px; color: var(--text-muted); margin-top:2px; line-height:1.4; }
    .tool-list { display:grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap:8px; }
    .muted-note { font-size:12px; color: var(--text-dim); line-height:1.5; }
    .summary-line { font-size:13px; color: var(--text-muted); }
    .summary-line strong { color:#E5E7EB; }
    .linklike { background:none; border:none; color:#34D399; cursor:pointer; font-size:12px; padding:0; text-decoration:underline; }
  </style>
</head>
<body>

  <!-- Spacecraft Header -->
  <header>
    <div class="brand">
      <div class="brand-icon">⚡</div>
      <div>
        <div class="brand-title">Botdigit BootForge</div>
        <div class="brand-subtitle">Universal Boot & Recovery Studio</div>
      </div>
    </div>

    <!-- Main Navigation Tabs -->
    <div class="nav-tabs">
      <button class="tab-btn active" onclick="switchTab('tabBoot')">🚀 Create Drive</button>
      <button class="tab-btn" onclick="switchTab('tabExplorer')">📁 Drive Files</button>
      <button class="tab-btn" onclick="switchTab('tabIsoIntel')">💿 Check an Image</button>
      <button class="tab-btn" onclick="switchTab('tabRepair')">🛠️ Repair Drive</button>
      <button class="tab-btn" onclick="switchTab('tabHardware')">🔎 This Mac</button>
    </div>
  </header>

  <main>
    <!-- Left Panel: Step 1 — choose the USB drive -->
    <div class="panel">
      <div class="panel-header">
        <div class="panel-title"><span class="step-num">1</span>Choose USB drive</div>
        <button class="btn-secondary" style="padding: 4px 10px; font-size: 11px;" onclick="loadDevices()">↻ Refresh</button>
      </div>

      <div id="deviceList" style="display: flex; flex-direction: column; gap: 10px;">
        <div class="muted-note" style="text-align: center; padding: 20px;">Looking for drives…</div>
      </div>

      <div id="protectedNote" class="muted-note"></div>

      <div class="muted-note" style="border-top: 1px solid var(--border); padding-top: 10px;">
        Plug in a USB stick or external SSD (8 GB – 2 TB). Your Mac's own disk and the drive BootForge runs from are locked and never shown here.
      </div>
    </div>

    <!-- Right Panel: Active Tab View -->
    <div class="content-area">

      <!-- Live OS Downloader Banner -->
      <div id="downloadBanner" style="display: none; background: rgba(6, 182, 212, 0.1); border: 1px solid rgba(6, 182, 212, 0.35); border-radius: 12px; padding: 14px 18px; box-shadow: 0 4px 20px var(--cyan-glow);">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
          <div style="font-weight: 700; font-size: 13px; color: #67E8F9; display: flex; align-items: center; gap: 8px;">
            <span>⬇️</span> <span id="downloadStatusText">Downloading OS Image...</span>
          </div>
          <div style="font-family: var(--font-mono); font-size: 11px; color: #34D399;" id="downloadSpeedText">⚡ Connecting...</div>
        </div>
        <div class="progress-track" style="height: 8px;">
          <div class="progress-bar-fill" id="downloadProgressBar" style="width: 0%;"></div>
        </div>
      </div>

      <!-- 🚀 TAB 1: CREATE DRIVE -->
      <div class="tab-content active" id="tabBoot">

        <!-- Step 2a: Botdigit Rescue Toolkit (preselected) -->
        <div class="panel">
          <div class="panel-header">
            <div>
              <div class="panel-title"><span class="step-num">2</span>Botdigit Rescue Toolkit <span class="badge badge-emerald" style="margin-left:8px;">Recommended</span></div>
              <div class="muted-note" style="margin-top: 4px;">Repair, backup, format and wipe tools every technician needs. Latest official versions, checksum-verified, downloaded once and cached on this Mac.</div>
            </div>
          </div>
          <div class="tool-list" id="toolkitList"><div class="muted-note">Loading tools…</div></div>
        </div>

        <!-- Step 2b: your own operating systems (optional) -->
        <div class="panel">
          <div class="panel-header">
            <div>
              <div class="panel-title">💿 Your operating systems <span class="muted-note" style="font-weight:400;">(optional)</span></div>
              <div class="muted-note" style="margin-top: 4px;">Windows, Linux, BSD installers… (.iso .wim .vhd .img). You can also copy files onto the drive later in Finder.</div>
            </div>
            <button class="btn-primary" style="font-size: 12px; padding: 8px 14px;" onclick="pickFromFinder()">➕ Add image…</button>
          </div>

          <div class="dropzone" id="dropzone" style="padding: 14px;" onclick="pickFromFinder()">
            <div style="font-size: 13px; color: var(--text-muted);">Click to choose image files in Finder (you can select several)</div>
          </div>

          <div class="cards-grid" id="isoCardsQueue"></div>
          <span id="selectedCount" style="display:none;">0</span>
        </div>

        <!-- Step 3: create -->
        <div class="panel">
          <div class="panel-header">
            <div class="panel-title"><span class="step-num">3</span>Create the drive</div>
          </div>

          <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 14px;">
            <div style="display: flex; flex-direction: column; gap: 6px;">
              <span class="muted-note" style="font-weight: 700;">WHAT TO DO</span>
              <select id="optSetupMode" class="btn-secondary" style="width: 100%;" onchange="updateSummary()">
                <option value="fresh_multiboot" selected>New drive — erase and make bootable</option>
                <option value="add_isos">Add to my existing BootForge drive (keep files)</option>
              </select>
            </div>
            <div style="display: flex; flex-direction: column; gap: 6px;">
              <span class="muted-note" style="font-weight: 700;">DRIVE NAME</span>
              <input type="text" id="optVolumeLabel" class="btn-secondary" value="BOTDIGIT" maxlength="11" style="font-family: var(--font-mono); font-weight: 700;">
            </div>
            <input type="hidden" id="optPartitionScheme" value="UNIVERSAL">
            <input type="hidden" id="optFilesystem" value="ExFAT">
          </div>

          <div class="summary-line" id="createSummary" style="margin-top: 12px;"></div>

          <button class="btn-primary" id="btnCreate" style="width: 100%; justify-content: center; margin-top: 12px; padding: 14px; font-size: 15px;" onclick="initiateBootDriveCreation()">
            🚀 Create BootForge drive
          </button>

          <div class="muted-note" style="margin-top: 10px;">
            Boots old PCs (Legacy BIOS), UEFI and Secure Boot PCs, 32-bit UEFI tablets and Intel Macs.
            Old PC shows a blinking cursor? <button class="linklike" onclick="injectLegacyBiosBootstrap()">Fix Legacy BIOS boot</button>
          </div>
        </div>

        <!-- Live Progress Box -->
        <div class="panel" id="progressBox" style="display: none;">
          <div class="panel-header">
            <div class="panel-title" id="progressPhase">Configuring Botdigit BootForge EFI Bootloader...</div>
            <span id="progressPercent" style="font-family: var(--font-mono); font-weight: 800; color: #34D399;">0%</span>
          </div>

          <div class="progress-track">
            <div class="progress-bar-fill" id="progressBar"></div>
          </div>

          <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 10px;">
            <div class="telemetry-card">
              <span class="telemetry-label">SPEED</span>
              <span class="telemetry-value" id="statSpeed">⚡ 0.0 MB/s</span>
            </div>
            <div class="telemetry-card">
              <span class="telemetry-label">DATA WRITTEN</span>
              <span class="telemetry-value" id="statCopied">0.0 / 0.0 GB</span>
            </div>
            <div class="telemetry-card">
              <span class="telemetry-label">ETA</span>
              <span class="telemetry-value" id="statEta">⏱️ --</span>
            </div>
            <div class="telemetry-card">
              <span class="telemetry-label">STREAM BUFFER</span>
              <span class="telemetry-value" style="color: #67E8F9;">4MB Active</span>
            </div>
          </div>

          <div class="terminal-box" id="terminalLog"></div>
        </div>
      </div>

      <!-- 🧰 TAB 2: RECOVERY CENTER -->
      <div class="tab-content" id="tabRecovery">
        <div class="panel">
          <div class="panel-header">
            <div>
              <div class="panel-title">🧰 Universal Recovery & Rescue Suite</div>
              <div style="font-size: 12px; color: var(--text-muted); margin-top: 2px;">Professional IT diagnostic and rescue toolkits ready to embed onto your USB</div>
            </div>
          </div>

          <div class="cards-grid" id="recoveryCatalogGrid">
            <!-- Populated via API -->
          </div>
        </div>
      </div>

      <!-- 📁 TAB 3: USB FILE EXPLORER -->
      <div class="panel tab-content" id="tabExplorer">
        <div class="panel-header">
          <div>
            <div class="panel-title">📁 USB Micro File Explorer</div>
            <div style="font-size: 12px; color: var(--text-muted); margin-top: 2px;" id="explorerMountBadge">Target: /Volumes/BOTDIGIT</div>
          </div>
          <div style="display: flex; gap: 8px;">
            <button class="btn-secondary" onclick="loadUsbFiles()">↻ Refresh</button>
            <button class="btn-primary" onclick="openSelectedMountInFinder()">📂 Open Finder</button>
          </div>
        </div>

        <div id="usbFileTree" style="display: flex; flex-direction: column; gap: 6px; max-height: 480px; overflow-y: auto;">
          <!-- Populated via JS -->
        </div>
      </div>

      <!-- 💿 TAB 4: ISO INTELLIGENCE -->
      <div class="panel tab-content" id="tabIsoIntel">
        <div class="panel-header">
          <div class="panel-title">💿 Automatic ISO Intelligence & Compatibility Engine</div>
        </div>

        <div style="display: flex; flex-direction: column; gap: 14px;">
          <div id="isoIntelDetails">
            <div style="color: var(--text-dim); font-size: 13px; text-align: center; padding: 30px;">
              Select or analyze an ISO in Boot Studio to view its deep El Torito header inspection, architecture compatibility scores, and kernel traits.
            </div>
          </div>
        </div>
      </div>

      <!-- 🔎 TAB 5: HARDWARE EXPLORER -->
      <div class="panel tab-content" id="tabHardware">
        <div class="panel-header">
          <div>
            <div class="panel-title">🔎 Host Hardware Telemetry & Architecture Explorer</div>
            <div style="font-size: 12px; color: var(--text-muted); margin-top: 2px;">Real-time system diagnostics and boot capability detection</div>
          </div>
          <button class="btn-secondary" onclick="loadHardwareTelemetry()">↻ Re-Scan</button>
        </div>

        <div class="telemetry-grid" id="hwGrid">
          <!-- Populated via API -->
        </div>
      </div>

      <!-- 🔥 TAB 6: 1-CLICK BOOT REPAIR -->
      <div class="panel tab-content" id="tabRepair">
        <div class="panel-header">
          <div>
            <div class="panel-title">🔥 1-Click Non-Destructive Boot Repair</div>
            <div style="font-size: 12px; color: var(--text-muted); margin-top: 2px;">Restores corrupted or missing EFI bootloader structures without touching or deleting your ISO files</div>
          </div>
        </div>

        <div style="background: rgba(245, 158, 11, 0.08); border: 1px solid rgba(245, 158, 11, 0.3); border-radius: 10px; padding: 16px; display: flex; flex-direction: column; gap: 8px;">
          <div style="font-size: 14px; font-weight: 700; color: #FCD34D;">🛡️ Non-Destructive Boot Structure Restoration</div>
          <div style="font-size: 13px; color: var(--text-muted);">
            If a PC shows a blank screen or fails to boot from this USB, BootForge will re-verify and reinstall <code>/EFI/BOOT/BOOTX64.EFI</code>, <code>grub.cfg</code>, and the <strong>Botdigit Cyber Emerald</strong> theme while preserving all ISOs in <code>/images/</code>.
          </div>
        </div>

        <div style="display: flex; gap: 12px; flex-wrap: wrap;">
          <button class="btn-primary" onclick="repairBootloader()">
            🛠️ REPAIR USB BOOTLOADER (UEFI + CONFIGS)
          </button>
          <button class="btn-secondary" onclick="injectLegacyBiosBootstrap()" style="border-color: #34D399; color: #34D399;">
            ⚡ FIX THINKPAD R400 / LEGACY BIOS BLINKING CURSOR
          </button>
        </div>

        <div class="terminal-box" id="repairLog" style="display: none;"></div>
      </div>

    </div>
  </main>

  <!-- Large Drive Safety Confirmation Modal -->
  <div class="modal-backdrop" id="safetyModal">
    <div class="modal-box">
      <div style="font-size: 18px; font-weight: 800; color: #FCD34D;">⚠️ Erase this drive?</div>
      <div id="safetyModalText" style="font-size: 13px; line-height: 1.6; color: var(--text-muted);"></div>
      
      <div style="background: rgba(239, 68, 68, 0.08); border: 1px solid rgba(239, 68, 68, 0.3); border-radius: 10px; padding: 14px;">
        <label style="display: flex; align-items: center; gap: 10px; cursor: pointer; font-size: 13px; font-weight: 600;">
          <input type="checkbox" id="safetyCheck">
          <span>Yes, erase everything on this drive.</span>
        </label>
      </div>

      <div style="display: flex; flex-direction: column; gap: 6px;">
        <span style="font-size: 12px; color: var(--text-dim);">To confirm, please type the device identifier <strong id="diskIdPrompt" style="color: #F8FAFC;"></strong>:</span>
        <input type="text" id="safetyInput" class="btn-secondary" placeholder="Type disk ID (e.g. disk8)">
      </div>

      <div style="display: flex; justify-content: flex-end; gap: 10px;">
        <button class="btn-secondary" onclick="closeSafetyModal()">Cancel</button>
        <button class="btn-danger" id="btnConfirmLargeDrive" disabled onclick="executeBootDriveCreation()">Erase and create</button>
      </div>
    </div>
  </div>

  <script>
    let devicesData = [];
    let selectedDevice = null;
    let selectedIsos = [];
    let toolkitItems = [];
    let selectedTools = new Set();

    // Initialize UI
    window.addEventListener('DOMContentLoaded', () => {
      loadDevices();
      loadToolkit();
      loadRecoveryCatalog();
      loadHardwareTelemetry();
    });

    function switchTab(tabId) {
      document.querySelectorAll('.tab-btn').forEach(btn => btn.classList.remove('active'));
      document.querySelectorAll('.tab-content').forEach(tab => tab.classList.remove('active'));
      
      event.currentTarget?.classList.add('active');
      const activeEl = document.getElementById(tabId);
      if (activeEl) activeEl.classList.add('active');

      if (tabId === 'tabExplorer') loadUsbFiles();
      if (tabId === 'tabHardware') loadHardwareTelemetry();
    }

    async function loadDevices() {
      try {
        const res = await fetch('/api/devices');
        devicesData = await res.json();
        renderDevices();
      } catch (e) {
        console.error("Failed to load devices", e);
      }
    }

    function renderDevices() {
      const list = document.getElementById('deviceList');
      const note = document.getElementById('protectedNote');
      list.innerHTML = '';
      const usable = devicesData.filter(d => !d.is_system_drive);
      const locked = devicesData.filter(d => d.is_system_drive);
      note.textContent = locked.length
        ? `🔒 ${locked.length} protected drive${locked.length > 1 ? 's' : ''} hidden (${locked.map(d => d.model).join(', ')}).`
        : '';

      if (usable.length === 0) {
        list.innerHTML = '<div class="muted-note" style="text-align: center; padding: 20px;">No USB drive found.<br>Plug one in and press Refresh.</div>';
        selectedDevice = null;
        updateSummary();
        return;
      }
      if (!usable.some(d => d.id === selectedDevice)) selectedDevice = usable[0].id;

      usable.forEach(dev => {
        const gb = (dev.capacity / 1e9).toFixed(1);
        const card = document.createElement('div');
        card.className = `device-card ${selectedDevice === dev.id ? 'selected' : ''}`;
        card.innerHTML = `
          <div class="device-header">
            <span class="device-name">${escapeHtml(dev.model || dev.vendor)}</span>
            <span class="device-capacity">${gb} GB</span>
          </div>
          <div class="device-meta"><span>/dev/${dev.id}</span></div>
          ${dev.capacity > 2.2e12 ? '<div class="muted-note" style="color:#FCD34D;">Larger than 2 TB: only the first 2 TB can be used.</div>' : ''}
        `;
        card.onclick = () => selectDevice(dev.id);
        list.appendChild(card);
      });
      updateSummary();
    }

    function selectDevice(id) {
      selectedDevice = id;
      renderDevices();
    }

    async function loadToolkit() {
      try {
        const res = await fetch('/api/toolkit');
        toolkitItems = await res.json();
        selectedTools = new Set(toolkitItems.filter(t => t.default_selected).map(t => t.id));
        renderToolkit();
      } catch (e) {
        document.getElementById('toolkitList').innerHTML = '<div class="muted-note">Could not load the toolkit list.</div>';
      }
    }

    function renderToolkit() {
      const list = document.getElementById('toolkitList');
      list.innerHTML = '';
      toolkitItems.forEach(t => {
        const row = document.createElement('label');
        row.className = 'tool-row';
        const size = t.approx_mb >= 1000 ? `${(t.approx_mb / 1000).toFixed(1)} GB` : `${t.approx_mb} MB`;
        row.innerHTML = `
          <input type="checkbox" ${selectedTools.has(t.id) ? 'checked' : ''}>
          <div>
            <div><span class="tool-name">${escapeHtml(t.name)}</span><span class="tool-size">${size}</span></div>
            <div class="tool-purpose">${escapeHtml(t.purpose)}</div>
          </div>`;
        row.querySelector('input').onchange = (e) => {
          if (e.target.checked) selectedTools.add(t.id); else selectedTools.delete(t.id);
          updateSummary();
        };
        list.appendChild(row);
      });
      updateSummary();
    }

    function updateSummary() {
      const el = document.getElementById('createSummary');
      const btn = document.getElementById('btnCreate');
      if (!el || !btn) return;
      const dev = devicesData.find(d => d.id === selectedDevice);
      const toolMb = toolkitItems.filter(t => selectedTools.has(t.id)).reduce((a, t) => a + t.approx_mb, 0);
      const isoMb = selectedIsos.reduce((a, i) => a + (i.size_bytes || 0) / 1e6, 0);
      const needGb = (toolMb + isoMb) / 1000;
      const mode = document.getElementById('optSetupMode').value;
      const parts = [];
      if (selectedTools.size) parts.push(`${selectedTools.size} rescue tool${selectedTools.size > 1 ? 's' : ''}`);
      if (selectedIsos.length) parts.push(`${selectedIsos.length} operating system${selectedIsos.length > 1 ? 's' : ''}`);
      const what = parts.length ? parts.join(' + ') : 'an empty bootable drive (copy images later)';

      if (!dev) {
        el.innerHTML = 'Plug in a USB drive to continue.';
        btn.disabled = true;
        return;
      }
      const capGb = Math.min(dev.capacity, 2.2e12) / 1e9;
      const fits = needGb < capGb * 0.97;
      el.innerHTML = `${mode === 'fresh_multiboot' ? '<strong style="color:#FCA5A5;">Erases</strong> ' : 'Adds to '}<strong>${escapeHtml(dev.model)}</strong> (${capGb.toFixed(0)} GB): ${what}` +
        ` — needs about <strong>${needGb.toFixed(1)} GB</strong>.` +
        (fits ? '' : ' <span style="color:#FCA5A5;">Not enough space: untick some tools or use a bigger drive.</span>') +
        (toolMb > 0 ? '<br><span class="muted-note">Tools are downloaded the first time only; later drives reuse the cached copies.</span>' : '');
      btn.disabled = !fits || (mode === 'add_isos' && !parts.length);
    }

    async function loadRecoveryCatalog() {
      try {
        const res = await fetch('/api/recovery-catalog');
        const tools = await res.json();
        const grid = document.getElementById('recoveryCatalogGrid');
        grid.innerHTML = '';

        tools.forEach(t => {
          const card = document.createElement('div');
          card.className = 'smart-card';
          card.innerHTML = `
            <div class="smart-card-top">
              <div class="card-icon">${t.icon}</div>
              <div>
                <div style="font-weight: 700; font-size: 15px;">${t.name}</div>
                <div style="font-size: 11px; color: var(--cyan); font-family: var(--font-mono);">${t.category} • ~${t.size_estimate}</div>
              </div>
            </div>
            <div style="font-size: 12px; color: var(--text-muted); line-height: 1.4;">${t.description}</div>
            <div style="display: flex; gap: 8px; margin-top: 4px;">
              <button class="btn-primary" style="flex: 1; justify-content: center; font-size: 12px; padding: 8px 12px;" onclick="startDownload('${t.id}')">
                ⬇️ Download ISO
              </button>
              <button class="btn-secondary" style="flex: 1; justify-content: center; font-size: 12px; padding: 8px 12px;" onclick="quickAdd('${t.name}', '${t.default_filename}', 'rescue')">
                📥 Add to Queue
              </button>
            </div>
          `;
          grid.appendChild(card);
        });
      } catch (e) {
        console.error("Failed to load recovery catalog", e);
      }
    }

    async function startDownload(itemId) {
      const banner = document.getElementById('downloadBanner');
      if (banner) banner.style.display = 'block';

      try {
        const res = await fetch('/api/start-download', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ item_id: itemId })
        });
        const r = await res.json();
        
        // Start polling download progress
        const poll = setInterval(async () => {
          const pRes = await fetch('/api/download-progress');
          const p = await pRes.json();
          if (p) {
            const bar = document.getElementById('downloadProgressBar');
            const txt = document.getElementById('downloadStatusText');
            const spd = document.getElementById('downloadSpeedText');
            if (bar) bar.style.width = `${p.percentage.toFixed(1)}%`;
            if (txt) txt.textContent = p.message || 'Downloading...';
            if (spd && p.speed_mbps > 0) spd.textContent = `⚡ ${p.speed_mbps.toFixed(1)} MB/s (ETA: ${p.eta_seconds}s)`;

            if (p.is_completed) {
              clearInterval(poll);
              if (bar) bar.style.width = '100%';
              if (txt) txt.textContent = `✅ ${p.file_name} downloaded successfully!`;
              // Add to selected queue
              quickAdd(p.file_name, p.file_name, 'rescue');
              setTimeout(() => { if (banner) banner.style.display = 'none'; }, 4000);
            }
          }
        }, 300);
      } catch (e) {
        alert("Download error: " + e.message);
      }
    }

    async function loadHardwareTelemetry() {
      try {
        const res = await fetch('/api/hardware-info');
        const hw = await res.json();
        
        document.getElementById('telemetryFirmware').textContent = hw.firmware_type;

        const grid = document.getElementById('hwGrid');
        grid.innerHTML = `
          <div class="telemetry-card">
            <span class="telemetry-label">PROCESSOR</span>
            <span class="telemetry-value">${hw.cpu_model}</span>
          </div>
          <div class="telemetry-card">
            <span class="telemetry-label">ARCHITECTURE</span>
            <span class="telemetry-value" style="color: #67E8F9;">${hw.cpu_architecture}</span>
          </div>
          <div class="telemetry-card">
            <span class="telemetry-label">CPU CORES</span>
            <span class="telemetry-value">${hw.cpu_cores} Cores</span>
          </div>
          <div class="telemetry-card">
            <span class="telemetry-label">SYSTEM MEMORY</span>
            <span class="telemetry-value" style="color: #34D399;">${hw.memory_gb} GB RAM</span>
          </div>
          <div class="telemetry-card">
            <span class="telemetry-label">FIRMWARE MODE</span>
            <span class="telemetry-value">${hw.firmware_type}</span>
          </div>
          <div class="telemetry-card">
            <span class="telemetry-label">SECURITY MODULE</span>
            <span class="telemetry-value">${hw.tpm_version}</span>
          </div>
          <div class="telemetry-card">
            <span class="telemetry-label">DISPLAY</span>
            <span class="telemetry-value">${hw.display_resolution}</span>
          </div>
          <div class="telemetry-card">
            <span class="telemetry-label">ACTIVE NETWORK</span>
            <span class="telemetry-value">${hw.network_interfaces.join(', ')}</span>
          </div>
        `;
      } catch (e) {
        console.error("Hardware telemetry error", e);
      }
    }

    async function testInVm(isUefi) {
      if (!selectedDevice) {
        alert("Please select a target USB drive on the left.");
        return;
      }
      try {
        const res = await fetch('/api/test-boot-vm', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ disk_id: selectedDevice, is_uefi: isUefi })
        });
        const r = await res.json();
        alert(r.message);
      } catch (e) {
        alert("VM test error: " + e.message);
      }
    }

    async function loadUsbFiles() {
      try {
        const mount = document.getElementById('optVolumeLabel').value || 'BOTDIGIT';
        const res = await fetch(`/api/usb-files?mount_path=/Volumes/${mount}`);
        const nodes = await res.json();
        const tree = document.getElementById('usbFileTree');
        tree.innerHTML = '';

        if (nodes.length === 0) {
          tree.innerHTML = `<div style="color: var(--text-dim); font-size: 13px; text-align: center; padding: 20px;">No files found in /Volumes/${mount}.</div>`;
          return;
        }

        function renderNode(n, depth = 0) {
          const div = document.createElement('div');
          div.style.paddingLeft = `${depth * 20}px`;
          div.style.display = 'flex';
          div.style.alignItems = 'center';
          div.style.justifyContent = 'space-between';
          div.style.padding = '8px 12px';
          div.style.background = 'var(--bg-card)';
          div.style.borderRadius = '6px';
          div.style.fontSize = '13px';
          div.style.fontFamily = 'var(--font-mono)';

          const icon = n.is_dir ? '📁' : (n.is_bootable_iso ? '💿' : '📄');
          div.innerHTML = `
            <div style="display: flex; align-items: center; gap: 8px;">
              <span>${icon}</span>
              <span style="font-weight: ${n.is_dir ? '700' : '400'}; color: ${n.is_bootable_iso ? '#34D399' : '#F8FAFC'}">${n.name}</span>
              ${n.category ? `<span class="badge badge-cyan">${n.category}</span>` : ''}
            </div>
            <div style="display: flex; align-items: center; gap: 12px;">
              <span style="color: var(--text-dim); font-size: 11px;">${n.is_dir ? '' : n.size_formatted}</span>
              ${!n.is_dir ? `<button class="btn-danger" style="padding: 2px 8px; font-size: 11px;" onclick="deleteFile('${n.relative_path}')">Delete</button>` : ''}
            </div>
          `;
          tree.appendChild(div);

          if (n.children) {
            n.children.forEach(child => renderNode(child, depth + 1));
          }
        }

        nodes.forEach(n => renderNode(n, 0));
      } catch (e) {
        console.error("Failed to load USB files", e);
      }
    }

    async function deleteFile(relPath) {
      if (!confirm(`Delete ${relPath} from USB drive?`)) return;
      const mount = `/Volumes/${document.getElementById('optVolumeLabel').value || 'BOTDIGIT'}`;
      try {
        const res = await fetch('/api/delete-usb-file', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ mount_path: mount, relative_path: relPath })
        });
        const resp = await res.json();
        alert(resp.message);
        loadUsbFiles();
      } catch (e) {
        alert("Delete failed: " + e.message);
      }
    }

    async function repairBootloader() {
      if (!selectedDevice) {
        alert("Please select a target USB drive on the left.");
        return;
      }
      const logEl = document.getElementById('repairLog');
      logEl.style.display = 'block';
      logEl.textContent = `[${new Date().toLocaleTimeString()}] Initiating Non-Destructive Boot Repair on /dev/${selectedDevice}...\n`;

      try {
        const res = await fetch('/api/repair-bootloader', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ disk_id: selectedDevice })
        });
        const r = await res.json();
        r.logs.forEach(l => {
          logEl.textContent += l + '\n';
        });
        alert(r.message);
      } catch (e) {
        logEl.textContent += `❌ Error: ${e.message}\n`;
      }
    }

    async function injectLegacyBiosBootstrap() {
      if (!selectedDevice) {
        alert("Please select a target USB drive on the left.");
        return;
      }

      const confirmed = confirm(
        `Fix Legacy BIOS boot on /dev/${selectedDevice}\n\n` +
        `Re-writes the BIOS boot code (sector 0 and the 1 MB gap before the first partition).\n` +
        `Partitions and image files are not touched.\n\n` +
        `macOS will ask for your administrator password. Proceed?`
      );

      if (!confirmed) return;

      try {
        const res = await fetch('/api/inject-legacy-mbr', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ disk_id: selectedDevice })
        });
        const data = await res.json();
        if (data.success) {
          alert("⚡ " + data.message);
        } else {
          alert("❌ Failed: " + data.message);
        }
      } catch (e) {
        alert("❌ Error communicating with BootForge engine: " + e.message);
      }
    }

    // Shortcut buttons: add a known image from the local tools folder. The file is analysed
    // for real; if it is not there you get a clear "File not found" message.
    function quickAdd(name, filename, category) {
      analyzeAndAddIso(`/Volumes/Mac2TB/Botdigit/tools/${filename}`);
    }

    function clearSelectedIsos() {
      selectedIsos = [];
      renderSelectedIsos();
    }

    function renderSelectedIsos() {
      document.getElementById('selectedCount').textContent = selectedIsos.length;
      const grid = document.getElementById('isoCardsQueue');
      grid.innerHTML = '';
      selectedIsos.forEach((iso, idx) => {
        const card = document.createElement('div');
        card.className = 'smart-card';
        card.innerHTML = `
          <div class="smart-card-top">
            <div style="flex: 1; min-width: 0;">
              <div style="font-weight: 700; font-size: 14px;">${escapeHtml(iso.display_name)}</div>
              <div style="font-size: 11px; color: var(--text-dim); font-family: var(--font-mono); overflow-wrap: anywhere;">${escapeHtml(iso.file_name)} • ${escapeHtml(iso.size || '')} • ${escapeHtml(iso.arch || '')}</div>
            </div>
            <button class="btn-secondary" style="padding: 4px 8px; font-size: 11px;" onclick="removeIso(${idx})" title="Remove">✕</button>
          </div>
          <button class="linklike" style="align-self:flex-start;" onclick="switchTab('tabIsoIntel')">Where will it boot? →</button>
        `;
        grid.appendChild(card);
      });
      updateSummary();
    }

    function removeIso(idx) {
      selectedIsos.splice(idx, 1);
      renderSelectedIsos();
    }

    async function pickFromFinder() {
      try {
        const res = await fetch('/api/pick-file');
        const data = await res.json();
        const paths = (data.paths && data.paths.length) ? data.paths : (data.path ? [data.path] : []);
        for (const p of paths) await analyzeAndAddIso(p);
      } catch (e) {
        console.error("Finder picker error", e);
      }
    }

    async function analyzeAndAddIso(path) {
      try {
        const res = await fetch('/api/analyze', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ path })
        });
        const data = await res.json();
        if (data.success) {
          selectedIsos.push({
            display_name: data.display_name,
            file_name: data.file_name,
            file_path: data.file_path,
            size: data.file_size_formatted,
            size_bytes: data.file_size_bytes,
            arch: data.arch
          });
          renderSelectedIsos();
          showIsoIntelligence(data);
        } else if (data.error) {
          alert(data.error);
        }
      } catch (e) {
        console.error("Analyze error", e);
      }
    }

    function showIsoIntelligence(data) {
      const details = document.getElementById('isoIntelDetails');
      details.innerHTML = `
        <div class="smart-card">
          <div style="font-weight: 800; font-size: 16px; color: #34D399;">${data.display_name}</div>
          <div style="font-size: 12px; color: var(--text-muted); font-family: var(--font-mono);">${data.file_name} (${data.file_size_formatted})</div>
          
          <div style="margin-top: 10px; display: flex; flex-direction: column; gap: 8px;">
            <div style="font-size: 12px; font-weight: 700; color: var(--text-dim);">ARCHITECTURE COMPATIBILITY AUDIT:</div>
            
            <div>
              <div style="display: flex; justify-content: space-between; font-size: 11px; font-family: var(--font-mono);">
                <span>UEFI x86_64</span>
                <span style="color: #34D399;">${data.uefi_x64_score || 100}%</span>
              </div>
              <div class="progress-track" style="height: 6px;"><div class="progress-bar-fill" style="width: ${data.uefi_x64_score || 100}%;"></div></div>
            </div>

            <div>
              <div style="display: flex; justify-content: space-between; font-size: 11px; font-family: var(--font-mono);">
                <span>Legacy BIOS x86</span>
                <span style="color: #34D399;">${data.bios_score || 100}%</span>
              </div>
              <div class="progress-track" style="height: 6px;"><div class="progress-bar-fill" style="width: ${data.bios_score || 100}%;"></div></div>
            </div>

            <div>
              <div style="display: flex; justify-content: space-between; font-size: 11px; font-family: var(--font-mono);">
                <span>Secure Boot</span>
                <span style="color: #FCD34D;">${data.secure_boot_score || 80}%</span>
              </div>
              <div class="progress-track" style="height: 6px;"><div class="progress-bar-fill" style="width: ${data.secure_boot_score || 80}%;"></div></div>
            </div>
          </div>
          ${(data.warnings || []).map(w => `<div style="margin-top:10px; font-size:12px; color:#FCD34D;">⚠️ ${escapeHtml(w)}</div>`).join('')}
          ${renderDeviceGuidance(data.device_guidance || [])}
        </div>
      `;
    }

    function escapeHtml(t) {
      return String(t).replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
    }

    // "Will it boot?" — one row per device class, steps expand on click.
    function renderDeviceGuidance(list) {
      if (!list.length) return '';
      const icon = v => v === 'Works' ? '✅' : (v === 'WorksWithSteps' ? '🟡' : '❌');
      const rows = list.map(t => `
        <details style="border-top:1px solid rgba(148,163,184,0.15); padding:6px 0;">
          <summary style="cursor:pointer; font-size:12px; list-style:none; display:flex; gap:8px;">
            <span>${icon(t.verdict)}</span>
            <span style="font-weight:700; min-width:0;">${escapeHtml(t.device_name)}</span>
          </summary>
          <div style="font-size:11px; color:var(--text-muted); margin:4px 0 0 24px;">${escapeHtml(t.summary)}</div>
          ${t.steps.length ? `<ol style="font-size:11px; margin:4px 0 0 24px; padding-left:16px; color:var(--text-dim);">${t.steps.map(st => `<li style="margin:2px 0;">${escapeHtml(st)}</li>`).join('')}</ol>` : ''}
        </details>`).join('');
      return `<div style="margin-top:12px;">
        <div style="font-size:12px; font-weight:700; color:var(--text-dim); margin-bottom:4px;">WILL IT BOOT? (tap a device for steps)</div>
        ${rows}
      </div>`;
    }

    function openSelectedMountInFinder() {
      const mount = `/Volumes/${document.getElementById('optVolumeLabel').value || 'BOTDIGIT'}`;
      fetch('/api/open-in-finder', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path: mount })
      });
    }

    function initiateBootDriveCreation() {
      if (!selectedDevice) {
        alert("Plug in a USB drive first.");
        return;
      }
      const dev = devicesData.find(d => d.id === selectedDevice);
      const mode = document.getElementById('optSetupMode').value;

      if (mode === 'fresh_multiboot') {
        openSafetyModal(dev);
      } else {
        executeBootDriveCreation();
      }
    }

    function openSafetyModal(dev) {
      const modal = document.getElementById('safetyModal');
      const text = document.getElementById('safetyModalText');
      const prompt = document.getElementById('diskIdPrompt');
      const gb = (dev.capacity / 1e9).toFixed(1);
      const mode = document.getElementById('optSetupMode').value;

      text.innerHTML = `<strong>${escapeHtml(dev.model)}</strong> (<code>/dev/${dev.id}</code>, ${gb} GB) will be <strong style="color:#FCA5A5;">completely erased</strong>. Everything on it is lost.<br><br>macOS will then ask for your password to write the boot code.`;
      document.getElementById('safetyCheck').checked = false;
      document.getElementById('safetyInput').value = '';
      document.getElementById('btnConfirmLargeDrive').disabled = true;
      prompt.textContent = dev.id;
      modal.style.display = 'flex';
    }

    function closeSafetyModal() {
      document.getElementById('safetyModal').style.display = 'none';
    }

    document.getElementById('safetyCheck').addEventListener('change', () => {
      const dev = devicesData.find(d => d.id === selectedDevice);
      const isChecked = document.getElementById('safetyCheck').checked;
      const inputVal = document.getElementById('safetyInput').value.trim();
      const btn = document.getElementById('btnConfirmLargeDrive');
      btn.disabled = !(dev && isChecked && inputVal === dev.id);
    });

    document.getElementById('safetyInput').addEventListener('input', () => {
      const dev = devicesData.find(d => d.id === selectedDevice);
      const isChecked = document.getElementById('safetyCheck').checked;
      const inputVal = document.getElementById('safetyInput').value.trim();
      const btn = document.getElementById('btnConfirmLargeDrive');
      btn.disabled = !(dev && isChecked && inputVal === dev.id);
    });

    async function executeBootDriveCreation() {
      closeSafetyModal();
      const box = document.getElementById('progressBox');
      const bar = document.getElementById('progressBar');
      const percent = document.getElementById('progressPercent');
      const phase = document.getElementById('progressPhase');
      const log = document.getElementById('terminalLog');
      const speedEl = document.getElementById('statSpeed');
      const copiedEl = document.getElementById('statCopied');
      const etaEl = document.getElementById('statEta');

      const mode = document.getElementById('optSetupMode').value;
      const scheme = document.getElementById('optPartitionScheme').value;
      const fs = document.getElementById('optFilesystem').value;
      const volumeLabel = document.getElementById('optVolumeLabel').value || 'BOTDIGIT';
      const createBtn = document.getElementById('btnCreate');
      createBtn.disabled = true;
      createBtn.textContent = '⏳ Working…';
      const finish = () => { createBtn.textContent = '🚀 Create BootForge drive'; updateSummary(); };

      box.style.display = 'block';
      box.scrollIntoView({ behavior: 'smooth' });
      bar.style.width = '0%';
      percent.textContent = '0%';

      log.textContent = `[${new Date().toLocaleTimeString()}] [Botdigit BootForge Studio v0.1.0]\n`;
      log.textContent += `[${new Date().toLocaleTimeString()}] Target: /dev/${selectedDevice} (${mode})\n`;

      let createdMountPoint = null;

      const pollTimer = setInterval(async () => {
        try {
          const res = await fetch('/api/transfer-progress');
          const p = await res.json();
          if (p) {
            phase.textContent = p.phase || 'Writing data blocks...';
            bar.style.width = `${p.percentage.toFixed(1)}%`;
            percent.textContent = `${Math.floor(p.percentage)}%`;

            if (p.speed_mbps > 0) speedEl.textContent = `⚡ ${p.speed_mbps.toFixed(1)} MB/s`;
            if (p.eta_seconds > 0) etaEl.textContent = `⏱️ ${p.eta_seconds}s`;

            if (p.logs) {
              const currentLogs = log.textContent;
              p.logs.forEach(l => {
                if (!currentLogs.includes(l)) {
                  log.textContent += l + '\n';
                  log.scrollTop = log.scrollHeight;
                }
              });
            }

            if (p.is_error) {
              clearInterval(pollTimer);
              phase.textContent = '❌ ' + (p.message || 'Failed');
              speedEl.textContent = '—';
              etaEl.textContent = '—';
              loadDevices();
              finish();
              return;
            }

            if (p.is_completed) {
              clearInterval(pollTimer);
              finish();
              bar.style.width = '100%';
              percent.textContent = '100%';
              phase.textContent = '✅ Drive ready — eject it and boot a computer from it (boot-menu key: F12 / F9 / F8 / F11, ⌥ on Intel Macs).';
              speedEl.textContent = '⚡ Completed';
              etaEl.textContent = '⏱️ 0s';
              loadDevices();
              if (createdMountPoint) setTimeout(() => openSelectedMountInFinder(), 600);
            }
          }
        } catch (e) {
          console.error("Poll error", e);
        }
      }, 150);

      try {
        const res = await fetch('/api/create-boot-drive', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            disk_id: selectedDevice,
            mode: mode,
            partition_scheme: scheme,
            filesystem: fs,
            volume_label: volumeLabel,
            isos: selectedIsos.map(i => ({ file_path: i.file_path, file_name: i.file_name, display_name: i.display_name })),
            toolkit: [...selectedTools]
          })
        });
        const resp = await res.json();
        if (resp.success) createdMountPoint = resp.mount_point;
      } catch (err) {
        log.textContent += `❌ Error: ${err.message}\n`;
        clearInterval(pollTimer);
        finish();
      }
    }
  </script>
</body>
</html>
"#;
