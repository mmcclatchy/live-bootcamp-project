document.addEventListener("DOMContentLoaded", () => {
    console.log("DOM fully loaded and parsed")

    const loginSection = document.getElementById("login-section");
    const twoFASection = document.getElementById("2fa-section");
    const signupSection = document.getElementById("signup-section");
    const passwordResetSection = document.getElementById("password-reset-section");
    const newPasswordSection = document.getElementById("new-password-section");

    const signupLink = document.getElementById("signup-link");
    const twoFALoginLink = document.getElementById("2fa-login-link");
    const signupLoginLink = document.getElementById("signup-login-link");
    const forgotPasswordLink = document.getElementById("forgot-password-link");
    const passwordResetLoginLink = document.getElementById("password-reset-login-link");

    if (!loginSection) console.error("Login section not found");
    if (!twoFASection) console.error("2FA section not found");
    if (!signupSection) console.error("Signup section not found");
    if (!passwordResetSection) console.error("Password reset section not found");
    if (!newPasswordSection) console.error("New password section not found");
    if (!signupLink) console.error("Signup link not found");
    if (!twoFALoginLink) console.error("2FA login link not found");
    if (!signupLoginLink) console.error("Signup login link not found");
    if (!forgotPasswordLink) console.error("Forgot password link not found");
    if (!passwordResetLoginLink) console.error("Password reset login link not found");

    function showSection(sectionToShow) {
        [loginSection, twoFASection, signupSection, passwordResetSection, newPasswordSection].forEach(section => {
            if (section) section.style.display = section === sectionToShow ? "block" : "none";
        });
    }

    if (signupLink) {
        signupLink.addEventListener("click", (e) => {
            e.preventDefault();
            showSection(signupSection);
        });
    }

    if (twoFALoginLink) {
        twoFALoginLink.addEventListener("click", (e) => {
            e.preventDefault();
            showSection(loginSection);
        });
    }

    if (signupLoginLink) {
        signupLoginLink.addEventListener("click", (e) => {
            e.preventDefault();
            showSection(loginSection);
        });
    }

    if (forgotPasswordLink) {
        forgotPasswordLink.addEventListener("click", (e) => {
            e.preventDefault();
            showSection(passwordResetSection);
        });
    }

    if (passwordResetLoginLink) {
        passwordResetLoginLink.addEventListener("click", (e) => {
            e.preventDefault();
            showSection(loginSection);
        });
    }

    // Check if there's a reset token in the URL
    const urlParams = new URLSearchParams(window.location.search);
    const resetToken = urlParams.get('token');
    if (resetToken) {
        document.getElementById('reset-token').value = resetToken;
        showSection(newPasswordSection);
    }

    // Login Form Handling
    const loginForm = document.getElementById("login-form");
    const loginButton = document.getElementById("login-form-submit");
    const loginErrAlert = document.getElementById("login-err-alert");

    if (loginButton) {
        loginButton.addEventListener("click", (e) => {
            e.preventDefault();

            const email = loginForm.email.value;
            const password = loginForm.password.value;

            fetch('/auth/login', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ email, password }),
            }).then(response => {
                if (response.status === 206) {
                    TwoFAForm.email.value = email;
                    response.json().then(data => {
                        TwoFAForm.login_attempt_id.value = data.loginAttemptId;
                    });

                    loginForm.email.value = "";
                    loginForm.password.value = "";

                    showSection(twoFASection);
                    loginErrAlert.style.display = "none";
                } else if (response.status === 200) {
                    loginForm.email.value = "";
                    loginForm.password.value = "";
                    loginErrAlert.style.display = "none";
                    alert("You have successfully logged in.");
                } else {
                    response.json().then(data => {
                        let error_msg = data.error;
                        if (error_msg !== undefined && error_msg !== null && error_msg !== "") {
                            loginErrAlert.innerHTML = `<span><strong>Error: </strong>${error_msg}</span>`;
                            loginErrAlert.style.display = "block";
                        } else {
                            loginErrAlert.style.display = "none";
                        }
                    });
                }
            });
        });
    }

    // Signup Form Handling
    const signupForm = document.getElementById("signup-form");
    const signupButton = document.getElementById("signup-form-submit");
    const signupErrAlert = document.getElementById("signup-err-alert");

    if (signupButton) {
        signupButton.addEventListener("click", (e) => {
            e.preventDefault();

            const email = signupForm.email.value;
            const password = signupForm.password.value;
            const requires2FA = signupForm.twoFA.checked;

            fetch('/auth/signup', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ email, password, requires2FA }),
            }).then(response => {
                if (response.ok) {
                    signupForm.email.value = "";
                    signupForm.password.value = "";
                    signupForm.twoFA.checked = false;
                    signupErrAlert.style.display = "none";
                    alert("You have successfully created a user.");
                    showSection(loginSection);
                } else {
                    response.json().then(data => {
                        let error_msg = data.error;
                        if (error_msg !== undefined && error_msg !== null && error_msg !== "") {
                            signupErrAlert.innerHTML = `<span><strong>Error: </strong>${error_msg}</span>`;
                            signupErrAlert.style.display = "block";
                        } else {
                            signupErrAlert.style.display = "none";
                        }
                    });
                }
            });
        });
    };

    // 2FA Form Handling
    const TwoFAForm = document.getElementById("2fa-form");
    const TwoFAButton = document.getElementById("2fa-form-submit");
    const TwoFAErrAlert = document.getElementById("2fa-err-alert");

    if (TwoFAButton) {
        TwoFAButton.addEventListener("click", (e) => {
            e.preventDefault();

            const email = TwoFAForm.email.value;
            const loginAttemptId = TwoFAForm.login_attempt_id.value;
            const TwoFACode = TwoFAForm.email_code.value;

            fetch('/auth/verify-2fa', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ email, loginAttemptId, "2FACode": TwoFACode }),
            }).then(response => {
                if (response.ok) {
                    TwoFAForm.email.value = "";
                    TwoFAForm.email_code.value = "";
                    TwoFAForm.login_attempt_id.value = "";
                    TwoFAErrAlert.style.display = "none";
                    alert("You have successfully logged in.");
                    showSection(loginSection);
                } else {
                    response.json().then(data => {
                        let error_msg = data.error;
                        if (error_msg !== undefined && error_msg !== null && error_msg !== "") {
                            TwoFAErrAlert.innerHTML = `<span><strong>Error: </strong>${error_msg}</span>`;
                            TwoFAErrAlert.style.display = "block";
                        } else {
                            TwoFAErrAlert.style.display = "none";
                        }
                    });
                }
            });
        });
    };

    // Password Reset Form Handling
    const passwordResetForm = document.getElementById("password-reset-form");
    const passwordResetButton = document.getElementById("password-reset-form-submit");
    const passwordResetErrAlert = document.getElementById("password-reset-err-alert");

    if (passwordResetButton) {
        passwordResetButton.addEventListener("click", (e) => {
            e.preventDefault();

            const email = passwordResetForm.email.value;

            fetch('/auth/initiate-password-reset', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ email }),
            }).then(response => {
                if (response.ok) {
                    passwordResetForm.email.value = "";
                    passwordResetErrAlert.style.display = "none";
                    alert("If the email exists, a password reset link has been sent.");
                    showSection(loginSection);
                } else {
                    response.json().then(data => {
                        let error_msg = data.error;
                        if (error_msg !== undefined && error_msg !== null && error_msg !== "") {
                            passwordResetErrAlert.innerHTML = `<span><strong>Error: </strong>${error_msg}</span>`;
                            passwordResetErrAlert.style.display = "block";
                        } else {
                            passwordResetErrAlert.style.display = "none";
                        }
                    });
                }
            });
        });
    }

    // New Password Form Handling
    const newPasswordForm = document.getElementById("new-password-form");
    const newPasswordButton = document.getElementById("new-password-form-submit");
    const newPasswordErrAlert = document.getElementById("new-password-err-alert");

    if (newPasswordButton) {
        newPasswordButton.addEventListener("click", (e) => {
            e.preventDefault();

            const token = newPasswordForm.token.value;
            const newPassword = newPasswordForm.new_password.value;
            const confirmPassword = newPasswordForm.confirm_password.value;

            if (newPassword !== confirmPassword) {
                newPasswordErrAlert.innerHTML = "<span><strong>Error: </strong>Passwords do not match</span>";
                newPasswordErrAlert.style.display = "block";
                return;
            }

            fetch('/auth/reset-password', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ token, new_password: newPassword }),
            }).then(response => {
                if (response.ok) {
                    newPasswordForm.reset();
                    newPasswordErrAlert.style.display = "none";
                    alert("Your password has been successfully reset.");
                    showSection(loginSection);
                } else {
                    response.json().then(data => {
                        let error_msg = data.error;
                        if (error_msg !== undefined && error_msg !== null && error_msg !== "") {
                            newPasswordErrAlert.innerHTML = `<span><strong>Error: </strong>${error_msg}</span>`;
                            newPasswordErrAlert.style.display = "block";
                        } else {
                            newPasswordErrAlert.style.display = "none";
                        }
                    });
                }
            });
        });
    }
});
