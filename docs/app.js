// BootForge Interactive Engine, Simulator & Download Center
document.addEventListener('DOMContentLoaded', () => {
  // 1. Generic Clipboard Copy Helper
  document.querySelectorAll('.copy-mini-btn, #cliCopyBtn').forEach(btn => {
    btn.addEventListener('click', async () => {
      const targetId = btn.dataset.copyTarget;
      let textToCopy = '';
      if (targetId) {
        const el = document.getElementById(targetId);
        if (el) textToCopy = el.innerText.trim();
      } else {
        const parentSnippet = btn.closest('.cmd-snippet, .cli-box');
        if (parentSnippet) {
          const codeEl = parentSnippet.querySelector('code, span:not(button)');
          if (codeEl) textToCopy = codeEl.innerText.trim();
        }
      }

      if (!textToCopy) return;

      try {
        await navigator.clipboard.writeText(textToCopy);
        const originalText = btn.innerText;
        btn.innerText = 'Copied!';
        btn.style.background = 'var(--accent-green)';
        btn.style.color = '#fff';
        setTimeout(() => {
          btn.innerText = originalText;
          btn.style.background = '';
          btn.style.color = '';
        }, 2000);
      } catch (err) {
        console.error('Copy failed:', err);
      }
    });
  });

  // 2. Download Tabs Switcher
  const tabBtns = document.querySelectorAll('.tab-btn');
  const tabContents = document.querySelectorAll('.tab-content');

  tabBtns.forEach(btn => {
    btn.addEventListener('click', () => {
      tabBtns.forEach(b => b.classList.remove('active'));
      tabContents.forEach(c => c.classList.remove('active'));

      btn.classList.add('active');
      const targetTab = document.getElementById(btn.dataset.tab);
      if (targetTab) {
        targetTab.classList.add('active');
      }
    });
  });

  // 3. Client OS Detection & Auto-Tab
  const userAgent = window.navigator.userAgent.toLowerCase();
  let defaultTab = 'tab-mac';

  if (userAgent.includes('win')) {
    defaultTab = 'tab-win';
  } else if (userAgent.includes('linux')) {
    defaultTab = 'tab-linux';
  } else if (userAgent.includes('mac')) {
    defaultTab = 'tab-mac';
  }

  const matchingTabBtn = document.querySelector(`.tab-btn[data-tab="${defaultTab}"]`);
  if (matchingTabBtn) {
    matchingTabBtn.click();
  }

  // 4. FAQ Accordion
  const faqItems = document.querySelectorAll('.faq-item');
  faqItems.forEach(item => {
    const q = item.querySelector('.faq-question');
    if (q) {
      q.addEventListener('click', () => {
        const isActive = item.classList.contains('active');
        faqItems.forEach(other => other.classList.remove('active'));
        if (!isActive) {
          item.classList.add('active');
        }
      });
    }
  });

  // 5. BootForge Studio Simulator
  const isoButtons = document.querySelectorAll('.iso-btn');
  const forgeBtn = document.getElementById('startForgeBtn');
  const logFeed = document.getElementById('simLogFeed');
  const progressBar = document.getElementById('simProgressFill');
  const progressText = document.getElementById('simProgressText');
  const speedText = document.getElementById('simSpeedText');
  const statusBadge = document.getElementById('simStatusBadge');

  let isForging = false;
  let selectedIso = {
    name: "Windows 11 Pro 24H2",
    size: "5.8 GB",
    arch: "x86_64 / ARM64",
    features: "TPM 2.0 Bypass + SecureBoot Fix + Auto-unattend"
  };

  isoButtons.forEach(btn => {
    btn.addEventListener('click', () => {
      if (isForging) return;
      isoButtons.forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      selectedIso = {
        name: btn.dataset.name || "Custom ISO",
        size: btn.dataset.size || "4.2 GB",
        arch: btn.dataset.arch || "x86_64",
        features: btn.dataset.features || "Standard UEFI Boot"
      };
      appendLog(`[SELECT] Switched target image payload to: ${selectedIso.name} (${selectedIso.size})`, 'cyan');
    });
  });

  function appendLog(message, type = 'info') {
    if (!logFeed) return;
    const line = document.createElement('div');
    line.className = `log-entry ${type}`;
    const time = new Date().toISOString().substring(11, 19);
    line.innerHTML = `<span style="opacity:0.5;">[${time}]</span> <span>${message}</span>`;
    logFeed.appendChild(line);
    logFeed.scrollTop = logFeed.scrollHeight;
  }

  if (forgeBtn) {
    forgeBtn.addEventListener('click', () => {
      if (isForging) return;
      runForgeSimulation();
    });
  }

  function runForgeSimulation() {
    isForging = true;
    forgeBtn.disabled = true;
    forgeBtn.innerText = '⚡ Forging Storage Media...';
    forgeBtn.style.opacity = '0.7';

    if (logFeed) logFeed.innerHTML = '';
    if (progressBar) progressBar.style.width = '0%';
    if (progressText) progressText.innerText = '0%';
    if (speedText) speedText.innerText = 'Analyzing...';
    if (statusBadge) {
      statusBadge.innerText = 'IN PROGRESS';
      statusBadge.style.color = 'var(--accent-orange)';
    }

    const steps = [
      { pct: 5, log: `[STORAGE] Acquiring direct descriptor handle: /dev/disk4 [SanDisk Extreme Pro 128GB]`, type: 'info', speed: 'N/A' },
      { pct: 15, log: `[SAFETY] Hardware heuristic check: Bus=USB 3.2, Removable=true, SystemDrive=false. PASSED.`, type: 'success', speed: 'N/A' },
      { pct: 25, log: `[IMAGE] Deep-analyzing ${selectedIso.name}: Found valid UDF 1.02 filesystem and EFI/BOOT/BOOTX64.EFI`, type: 'cyan', speed: 'N/A' },
      { pct: 38, log: `[PARTITION] Creating hybrid GPT layout: ESP (FAT32, 100MB) + Data (ExFAT, 119GB) + Doctor OS (850MB)`, type: 'info', speed: '380 MB/s' },
      { pct: 55, log: `[BOOTLOADER] Compiling BootForge UEFI dynamic chainloader with Ventoy & Limine fallback stubs`, type: 'active', speed: '420 MB/s' },
      { pct: 72, log: `[STREAM] Direct block-level zero-copy payload transfer: 100% of ${selectedIso.size} written`, type: 'active', speed: '465 MB/s' },
      { pct: 88, log: `[HASH] Verifying SHA-256 block digest across 11,324 chunks... MATCHED.`, type: 'success', speed: '440 MB/s' },
      { pct: 95, log: `[QEMU] Spawning headless QEMU virtual test harness to test UEFI handoff...`, type: 'cyan', speed: 'Testing' },
      { pct: 100, log: `[SUCCESS] Virtual boot test passed in 1.2s! Media is ready for deployment.`, type: 'success', speed: 'Complete' }
    ];

    let currentStep = 0;

    function nextStep() {
      if (currentStep < steps.length) {
        const item = steps[currentStep];
        if (progressBar) progressBar.style.width = `${item.pct}%`;
        if (progressText) progressText.innerText = `${item.pct}%`;
        if (speedText) speedText.innerText = item.speed;
        appendLog(item.log, item.type);
        currentStep++;
        setTimeout(nextStep, 600);
      } else {
        isForging = false;
        forgeBtn.disabled = false;
        forgeBtn.innerText = '✓ Storage Forged Successfully! (Run Again)';
        forgeBtn.style.opacity = '1';
        if (statusBadge) {
          statusBadge.innerText = 'VERIFIED BOOTABLE';
          statusBadge.style.color = 'var(--accent-green)';
        }
      }
    }

    nextStep();
  }
});
