<template>
  <Transition name="splash-fade">
    <div v-if="visible" class="splash-overlay">
      <!-- Animated background grid -->
      <div class="splash-bg-grid"></div>
      <!-- Glow effect behind logo -->
      <div class="splash-glow"></div>

      <!-- Logo and text -->
      <div class="splash-content">
        <div class="splash-logo-wrapper" :class="{ 'logo-animate': logoVisible }">
          <img
            src="@/assets/images/splash-icon.png"
            alt="LapAI"
            class="splash-logo"
            @load="onLogoLoad"
          />
          <!-- Circuit lines decoration -->
          <div class="circuit-ring circuit-ring-1"></div>
          <div class="circuit-ring circuit-ring-2"></div>
          <div class="circuit-dots">
            <span v-for="i in 8" :key="i" class="circuit-dot" :style="`--i:${i}`"></span>
          </div>
        </div>

        <!-- App name -->
        <div class="splash-name" :class="{ 'name-animate': logoVisible }">
          <span class="name-lap">Lap</span><span class="name-ai">AI</span>
        </div>

        <!-- Tagline -->
        <p class="splash-tagline" :class="{ 'tagline-animate': logoVisible }">
          AI-powered image management
        </p>

        <!-- Loading dots -->
        <div class="splash-loading" :class="{ 'loading-animate': logoVisible }">
          <span v-for="i in 3" :key="i" class="loading-dot" :style="`--i:${i}`"></span>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup>
import { ref, onMounted } from 'vue'

const props = defineProps({
  duration: { type: Number, default: 2200 }
})

const emit = defineEmits(['done'])
const visible = ref(true)
const logoVisible = ref(false)

function onLogoLoad() {
  // Start animations after logo loads
  requestAnimationFrame(() => {
    logoVisible.value = true
  })
}

onMounted(() => {
  // Fallback: start animation after 100ms even if load event doesn't fire
  setTimeout(() => {
    logoVisible.value = true
  }, 100)

  // Hide splash after duration
  setTimeout(() => {
    visible.value = false
    setTimeout(() => emit('done'), 600)
  }, props.duration)
})
</script>

<style scoped>
.splash-overlay {
  position: fixed;
  inset: 0;
  z-index: 9999;
  background: #000;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

/* Animated background grid */
.splash-bg-grid {
  position: absolute;
  inset: 0;
  background-image:
    linear-gradient(rgba(255,255,255,0.03) 1px, transparent 1px),
    linear-gradient(90deg, rgba(255,255,255,0.03) 1px, transparent 1px);
  background-size: 40px 40px;
  animation: gridPulse 3s ease-in-out infinite;
}

@keyframes gridPulse {
  0%, 100% { opacity: 0.4; }
  50% { opacity: 1; }
}

/* Central glow */
.splash-glow {
  position: absolute;
  width: 500px;
  height: 500px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(255,255,255,0.08) 0%, rgba(180,180,255,0.04) 40%, transparent 70%);
  animation: glowPulse 2s ease-in-out infinite;
}

@keyframes glowPulse {
  0%, 100% { transform: scale(0.95); opacity: 0.7; }
  50% { transform: scale(1.05); opacity: 1; }
}

/* Main content wrapper */
.splash-content {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

/* Logo wrapper */
.splash-logo-wrapper {
  position: relative;
  width: 160px;
  height: 160px;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transform: scale(0.6) translateY(20px);
  transition: opacity 0.6s cubic-bezier(0.34, 1.56, 0.64, 1),
              transform 0.6s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.splash-logo-wrapper.logo-animate {
  opacity: 1;
  transform: scale(1) translateY(0);
}

.splash-logo {
  width: 140px;
  height: 140px;
  border-radius: 28px;
  object-fit: cover;
  filter: drop-shadow(0 0 30px rgba(255,255,255,0.3)) drop-shadow(0 0 60px rgba(200,200,255,0.15));
  animation: logoFloat 3s ease-in-out infinite;
}

@keyframes logoFloat {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-6px); }
}

/* Circuit rings around logo */
.circuit-ring {
  position: absolute;
  border-radius: 50%;
  border: 1px solid rgba(255,255,255,0.12);
  animation: ringRotate 8s linear infinite;
}

.circuit-ring-1 {
  width: 180px;
  height: 180px;
  border-color: rgba(255,255,255,0.1);
  border-style: dashed;
  animation-duration: 12s;
}

.circuit-ring-2 {
  width: 210px;
  height: 210px;
  border-color: rgba(180,180,255,0.08);
  animation-direction: reverse;
  animation-duration: 20s;
}

@keyframes ringRotate {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* Circuit dots */
.circuit-dots {
  position: absolute;
  width: 200px;
  height: 200px;
  animation: ringRotate 8s linear infinite;
}

.circuit-dot {
  position: absolute;
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: rgba(255,255,255,0.6);
  top: 50%;
  left: 50%;
  transform-origin: 0 0;
  transform: rotate(calc(var(--i) * 45deg)) translateX(98px) translateY(-2px);
  box-shadow: 0 0 6px rgba(255,255,255,0.8);
}

/* App name */
.splash-name {
  font-family: 'Inter', 'Segoe UI', sans-serif;
  font-size: 32px;
  font-weight: 800;
  letter-spacing: 2px;
  opacity: 0;
  transform: translateY(10px);
  transition: opacity 0.5s ease 0.3s, transform 0.5s ease 0.3s;
}

.splash-name.name-animate {
  opacity: 1;
  transform: translateY(0);
}

.name-lap {
  color: #fff;
  font-weight: 800;
}

.name-ai {
  color: rgba(180, 180, 255, 0.9);
  font-weight: 300;
}

/* Tagline */
.splash-tagline {
  font-family: 'Inter', 'Segoe UI', sans-serif;
  font-size: 11px;
  font-weight: 400;
  letter-spacing: 3px;
  text-transform: uppercase;
  color: rgba(255,255,255,0.3);
  opacity: 0;
  transform: translateY(8px);
  transition: opacity 0.5s ease 0.5s, transform 0.5s ease 0.5s;
}

.splash-tagline.tagline-animate {
  opacity: 1;
  transform: translateY(0);
}

/* Loading dots */
.splash-loading {
  display: flex;
  gap: 8px;
  margin-top: 8px;
  opacity: 0;
  transition: opacity 0.4s ease 0.7s;
}

.splash-loading.loading-animate {
  opacity: 1;
}

.loading-dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: rgba(255,255,255,0.5);
  animation: dotPulse 1.2s ease-in-out infinite;
  animation-delay: calc(var(--i) * 0.2s);
}

@keyframes dotPulse {
  0%, 100% { transform: scale(0.6); opacity: 0.3; }
  50% { transform: scale(1.2); opacity: 1; }
}

/* Fade transition */
.splash-fade-leave-active {
  transition: opacity 0.6s ease, transform 0.6s ease;
}

.splash-fade-leave-to {
  opacity: 0;
  transform: scale(1.05);
}
</style>
