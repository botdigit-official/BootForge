// BootForge Interactive Engine & Web Simulator
document.addEventListener('DOMContentLoaded', () => {
  // 1. Copy CLI command helper
  const copyBtn = document.getElementById('cliCopyBtn');
  const cliText = document.getElementById('cliText');
  if (copyBtn && cliText) {
    copyBtn.addEventListener('click', async () => {
      try {
        await navigator.clipboard.writeText(cliText.innerText.trim());
        const originalText = copyBtn.innerText;
        copyBtn.innerText = 'Copied!';
        copyBtn.style.background = 'var(--accent-green)';
        setTimeout(() => {
          copyBtn.innerText = originalText;
          copyBtn.style.background = '';
        }, 2000);
      } catch (err) {
        console.error('Clipboard copy failed', err);
      }
    });
  }

  // 2. FAQ Accordion
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

  // 3. Client OS Detection
  const userAgent = window.navigator.userAgent.toLowerCase();
  const macCard = document.getElementById('dl-mac');
  const winCard = document.getElementById('dl-win');
  const linuxCard = document.getElementById('dl-linux');

  if (userAgent.includes('mac')) {
    macCard?.classList.add('highlight');
  } else if (userAgent.includes('win')) {
    winCard?.classList.add('highlight');
    macCard?.classList.remove('highlight');
  } else if (userAgent.includes('linux')) {
    linuxCard?.classList.add('highlight');
    macCard?.classList.remove('highlight');
  }

  // 4. BootForge Studio Simulator
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
      appendLog(`[SELECT] Switched target image to: ${selectedIso.name} (${selectedIso.size})`, 'cyan');
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
    forgeBtn.innerText = '⚡ Forging Media...';
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
      { pct: 5, log: `[ANALYZE] Opening storage handle to /dev/disk4 [SanDisk Extreme Pro 128GB]`, type: 'info', speed: 'N/A' },
      { pct: 15, log: `[SAFETY] Verifying anti-brick heuristics: Disk is removable, not system volume. PASSED.`, type: 'success', speed: 'N/A' },
      { pct: 25, log: `[IMAGE] Deep-analyzing ${selectedIso.name}: Found valid UDF 1.02 filesystem and EFI/BOOT/BOOTX64.EFI`, type: 'cyan', speed: 'N/A' },
      { pct: 35, log: `[PARTITION] Creating hybrid GPT layout: EFI ESP (FAT32, 100MB) + Payloads (ExFAT, 119GB)`, type: 'info', speed: '380 MB/s' },
      { pct: 55, log: `[BOOTLOADER] Generating BootForge UEFI bootloader chain with secure stub injection`, type: 'active', speed: '420 MB/s' },
      { pct: 75, log: `[STREAM] Direct block-level zero-copy payload transfer: 100% of ${selectedIso.size} committed`, type: 'active', speed: '450 MB/s' },
      { pct: 88, log: `[HASH] Verifying cryptographic SHA-256 block digest... MATCHED.`, type: 'success', speed: '410 MB/s' },
      { pct: 95, log: `[QEMU] Spawning headless QEMU virtual test harness to test UEFI handoff...`, type: 'cyan', speed: 'Testing' },
      { pct: 100, log: `[SUCCESS] Virtual boot test passed in 1.4s! Drive is ready for deployment.`, type: 'success', speed: 'Complete' }
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
        setTimeout(nextStep, 650);
      } else {
        isForging = false;
        forgeBtn.disabled = false;
        forgeBtn.innerText = '✓ Drive Forged Successfully! (Run Again)';
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
