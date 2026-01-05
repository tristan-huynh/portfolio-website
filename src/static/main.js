let isClosing = false;
let activeElement = null;
let turnstileLoaded = false;

const mobileQuery = window.matchMedia("(max-width: 768px)");

function showMobilePopup(content) {
    if (isClosing) {
        return;
    }
    const isMobile = navigator.userAgentData?.mobile || mobileQuery.matches;
    const popup = document.getElementById('mobile-popup');
    if (popup.classList.contains('flex')) {
        return;
    }
    if (isMobile) {
        const mobileContent = document.getElementById('mobile-detailed-content');
        mobileContent.innerHTML = content;
        popup.classList.remove('hidden');
        popup.classList.add('flex');
    }
}

function closeMobilePopup() {
    isClosing = true;
    const popup = document.getElementById('mobile-popup');
    popup.classList.add('hidden');
    popup.classList.remove('flex');
    setTimeout(() => {
        isClosing = false;
    }, 300);
}

function updateContent(detailed, isLink = false) {
    // disable hover on mobile
    if (mobileQuery.matches && !isLink) {
        return;
    }

    const detailedBlock = document.getElementById("detailed-content");
    if (detailedBlock) {
        detailedBlock.style.opacity = '0';
        setTimeout(() => {
            detailedBlock.innerHTML = detailed;
            detailedBlock.style.opacity = '1';
        }, 150);
    }
    if (!isLink) {
        showMobilePopup(detailed);
    }
}

function updateContentImg(img, detailed) {
    // disable hover on mobile
    if (mobileQuery.matches) {
        return;
    }

    const detailedBlock = document.getElementById("detailed-content");
    if (detailedBlock) {
        detailedBlock.style.opacity = '0';
        setTimeout(() => {
            detailedBlock.innerHTML = detailed + '<br><img id="img-content" src="' + img + '" alt="Image" class="mt-2 w-full h-auto border-4 border-blue-800 p-2"/>';
            detailedBlock.style.opacity = '1';
        }, 150);
    }
    const imgContent = detailed + '<br><img id="img-content" src="' + img + '" alt="Image" class="mt-2 w-full h-auto border-4 border-blue-800 p-2"/>';
    showMobilePopup(imgContent);
}

function clearContent() {
    // disable hover on mobile
    if (mobileQuery.matches) {
        return;
    }

    const detailedBlock = document.getElementById("detailed-content");
    if (detailedBlock) {
        detailedBlock.style.opacity = '0';
        setTimeout(() => {
            detailedBlock.innerHTML = "Hover over highlighted items to see more details here.";
            detailedBlock.style.opacity = '1';
        }, 150);
    }
}

function initHoverInteractions() {
    const hoverElements = document.querySelectorAll('[data-hover]');

    hoverElements.forEach(element => {
        element.addEventListener('mouseenter', function () {
            const hoverText = this.getAttribute('data-hover');
            const hoverImg = this.getAttribute('data-hover-img');
            const isLink = this.getAttribute('data-is-link') === 'true';

            if (hoverImg) {
                updateContentImg(hoverImg, hoverText);
            } else {
                updateContent(hoverText, isLink);
            }
        });

        element.addEventListener('mouseleave', function () {
            clearContent();
        });
    });
}

function initMobileInteractions() {
    if (mobileQuery.matches) {
        const hoverElements = document.querySelectorAll('[data-hover]:not(a[href])');
        hoverElements.forEach(element => {
            element.addEventListener('click', function (e) {
                if (this !== activeElement) {
                    e.preventDefault();

                    const hoverText = this.getAttribute('data-hover');
                    const hoverImg = this.getAttribute('data-hover-img');

                    if (hoverImg) {
                        const imgContent = hoverText + '<br><img id="img-content" src="' + hoverImg + '" alt="Image" class="mt-2 w-full h-auto border-4 border-blue-800 p-2"/>';
                        showMobilePopup(imgContent);
                    } else {
                        showMobilePopup(hoverText);
                    }

                    activeElement = this;
                }
            });
        });
    }
}

function initActionHandlers() {
    // click events with data-action attributes
    document.addEventListener('click', function (e) {
        const target = e.target.closest('[data-action]');
        if (target) {
            const action = target.getAttribute('data-action');

            // prevent event bubbling for specific actions
            if (action === 'closeMobilePopup') {
                e.stopPropagation();
            }

            switch (action) {
                case 'openContactModal':
                    openContactModal();
                    break;
                case 'closeContactModal':
                    closeContactModal();
                    break;
                case 'submitContactForm':
                    submitContactForm();
                    break;
                case 'closeSuccessModal':
                    closeSuccessModal();
                    break;
                case 'closeMobilePopup':
                    closeMobilePopup();
                    break;
            }
        }
    });
}

async function openContactModal() {
    const modal = document.getElementById('contact-modal');
    if (!modal) return;

    try {
        await loadTurnstile();
    } catch (error) {
        console.error('Failed to load Turnstile:', error);
    }

    modal.classList.remove('hidden');
    modal.classList.add('flex');
}

function closeContactModal() {
    document.getElementById('contact-modal').classList.add('hidden');
    document.getElementById('contact-form').reset();
    clearFormErrors();
    if (window.turnstile) {
        turnstile.reset();
    }
}

function closeSuccessModal() {
    document.getElementById('success-modal').classList.add('hidden');
    document.getElementById('contact-modal').classList.add('hidden');
    document.getElementById('contact-form').reset();
    if (window.turnstile) {
        turnstile.reset();
    }
}

function clearFormErrors() {
    const errorElements = ['name-error', 'email-error', 'message-error', 'form-error'];
    errorElements.forEach(id => {
        const element = document.getElementById(id);
        if (element) {
            element.textContent = '';
            element.classList.add('hidden');
        }
    });
}

function showFormError(message) {
    const errorElement = document.getElementById('form-error');
    if (errorElement) {
        errorElement.textContent = message;
        errorElement.classList.remove('hidden');
    }
}

function loadTurnstile() {
    if (turnstileLoaded) return Promise.resolve();

    return new Promise((resolve, reject) => {
        const script = document.createElement('script');
        script.src = 'https://challenges.cloudflare.com/turnstile/v0/api.js';
        script.async = true;
        script.defer = true;
        script.onload = () => {
            turnstileLoaded = true;
            resolve();
        };
        script.onerror = reject;
        document.head.appendChild(script);
    });
}

async function submitContactForm() {
    clearFormErrors();

    const form = document.getElementById('contact-form');
    const formData = new FormData(form);

    try {
        const response = await fetch('/contact', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/x-www-form-urlencoded',
            },
            body: new URLSearchParams(formData)
        });

        if (response.ok) {
            document.getElementById('success-modal').classList.remove('hidden');
            document.getElementById('success-modal').classList.add('flex');
        } else if (response.status === 400) {
            const errors = await response.json();
            showFormError(errors.message || 'Verification failed. Please try again.');
        } else if (response.status === 500) {
            const errors = await response.json();
            showFormError(errors.message || 'Server error. Please try again.');
        } else {
            showFormError('An error occurred.');
            console.error('Unexpected response:', response);
        }
    } catch (error) {
        showFormError('Network error. Please check your connection.');
    }
}

// initialize all event listeners when DOM is ready
document.addEventListener('DOMContentLoaded', function () {
    initHoverInteractions();
    initMobileInteractions();
    initActionHandlers();
});

// reinitialize mobile interactions on viewport change
mobileQuery.addEventListener('change', initMobileInteractions);
