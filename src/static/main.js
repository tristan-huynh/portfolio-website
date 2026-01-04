let isClosing = false;

function showMobilePopup(content) {
    if (isClosing) {
        return;
    }
    const isMobile = navigator.userAgentData?.mobile || window.innerWidth < 768;
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
    const detailedBlock = document.getElementById("detailed-content");
    if (detailedBlock) {
        detailedBlock.style.opacity = '0';
        setTimeout(() => {
            detailedBlock.innerHTML = detailed + '<br><img id="img-content" src="' + img + '" alt="Image" class="mt-2 w-full h-auto border-4 border-blue-800 p-2"/>';
            detailedBlock.style.opacity = '1';
        }, 150);
    }
    showMobilePopup(detailed + '<br><img id="img-content" src="' + img + '" alt="Image" class="mt-2 w-full h-auto border-4 border-blue-800 p-2"/>');
}
function clearContent() {
    const detailedBlock = document.getElementById("detailed-content");
    if (detailedBlock) {
        detailedBlock.style.opacity = '0';
        setTimeout(() => {
            detailedBlock.innerHTML = "Hover over highlighted items to see more details here.";
            detailedBlock.style.opacity = '1';
        }, 150);
    }
}

let isMobile = window.matchMedia("(max-width: 768px)").matches;
let activeElement = null;
if (isMobile) {
    document.addEventListener('DOMContentLoaded', function () {
        const hoverElements = document.querySelectorAll('[onmouseenter]:not(a[href])');
        hoverElements.forEach(element => {
            element.addEventListener('click', function (e) {
                if (this !== activeElement) {
                    e.preventDefault();
                    const mouseenterAttr = this.getAttribute('onmouseenter');
                    eval(mouseenterAttr);
                    activeElement = this;
                }
            });
        });
    });
}

function openContactModal() {
    const modal = document.getElementById('contact-modal');
    if (!modal) return;
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

function showFieldError(fieldId, message) {
    const errorElement = document.getElementById(`${fieldId}-error`);
    if (errorElement) {
        errorElement.textContent = message;
        errorElement.classList.remove('hidden');
    }
}

function showFormError(message) {
    const errorElement = document.getElementById('form-error');
    if (errorElement) {
        errorElement.textContent = message;
        errorElement.classList.remove('hidden');
    }
}


async function submitContactForm() {
    clearFormErrors();
    
    const form = document.getElementById('contact-form');
    const formData = new FormData(form);
    
    try {
        const response = await fetch('/contact', {
            method: 'POST',
            body: formData
        });
        
        if (response.ok) {
            document.getElementById('success-modal').classList.remove('hidden');
            document.getElementById('success-modal').classList.add('flex');
        } else if (response.status === 400) {
            const errors = await response.json();
            showFormError(errors.message || 'Verification failed. Please try again.');
        } else if (response.status === 500) {
            const errors = await response.json();
            showFormError(errors.message || 'Server error. Please try again later.');
        } else {
            showFormError('An error occurred. Please try again later.');
        }
    } catch (error) {
        console.error('Form submission error:', error);
        showFormError('Network error. Please check your connection.');
    }
}