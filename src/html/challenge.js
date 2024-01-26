// Requires IMHUMANE_API_URL to be defined earlier

// From SO: https://stackoverflow.com/a/18650249
// Handles prepending the MIME type too.
function blobToBase64(blob) {
    return new Promise((resolve, _) => {
        const reader = new FileReader();
        reader.onloadend = () => resolve(reader.result);
        reader.readAsDataURL(blob);
    });
}

function newElement(typ, cssClass) {
    const elem = document.createElement(typ);
    elem.classList.add(cssClass);
    return elem;
}

function asleep(delay) {
    return new Promise((resolve) => setTimeout(resolve, delay));
}

async function fetchChallenge() {
    const response = await fetch(IMHUMANE_API_URL, {
        method: "GET",
    });
    const image = await blobToBase64(await response.blob());
    return new Challenge(response.headers, image);
}

class Challenge {
    constructor(
        headers,
        base64Image
    ) {
        this.challengeId = headers.get("X-Imhumane-Id");
        this.topic = headers.get("X-Imhumane-Topic");
        this.gapSize = +headers.get("X-Imhumane-Gap-Size");
        this.imageSize = +headers.get("X-Imhumane-Image-Size");
        this.gridLength = +headers.get("X-Imhumane-Grid-Length");
        this.imageUrl = `url("${base64Image}")`;
    }

    /**
     * Validate the users's answer
     * @param {String} answer
     */
    async validate(answer) {
        const body = JSON.stringify({ answer, challenge_id: this.challengeId });
        const response = await fetch(IMHUMANE_API_URL, {
            method: "POST",
            body,
            headers: {
                'Content-Type': 'application/json'
            }
        });
        if (response.status == 204) {
            return true;
        }
        return false;
    }
}

class ChallengeGrid {
    /**
     *
     * @param {ChallengeContainer} container
     * @param {Challenge} challenge
     */
    constructor(container, challenge) {
        this.container = container;
        this.challenge = challenge;

        const cssClass = container.cssClass;
        const { gapSize, imageSize, gridLength } = challenge;

        // Elements
        this.title = newElement("p", "imhumane-title");
        this.title.innerHTML = `Select all images containing <br /><b>${challenge.topic}</b>`;

        this.checkboxElements = [];
        for (let i = 1; i <= gridLength ** 2; i++) {
            const elem = newElement("input", "imhumane-checkbox");
            elem.type = "checkbox";
            this.checkboxElements.push(elem);
        }

        this.grid = newElement("div", "imhumane-grid");
        this.checkboxElements.forEach(elem => this.grid.appendChild(elem));

        this.actions = newElement("span", "imhumane-actions");
        this.button = newElement("button", "imhumane-action-submit");
        this.button.innerText = "Submit";
        this.button.type = "button";
        this.actions.appendChild(this.button);

        // Disable button on click
        this.button.addEventListener("click", () => {
            this.button.disabled = true;
        });

        // Styling
        const containerLength = gapSize + ((gapSize + imageSize) * gridLength);
        const gapPercentage = (((gapSize * (gridLength + 1)) / containerLength) / 4) * 100;

        this.cssStyle = `
            .${this.cssClass}:not(.imhumane-responsive) div {
                width: ${this.containerLength}px;
            }

            .${cssClass} .imhumane-body > * {
                gap: ${gapPercentage}%;
                padding: ${gapPercentage}%;
            }

            .${cssClass}:not(.imhumane-responsive) .imhumane-body > * {
                width: ${containerLength}px;
                /* Percentage gaps are based on the parent element's width.
                To avoid weird rendering, use px when not responsive.    */
                gap: ${gapSize}px;
                padding: ${gapSize}px;
            }

            .${cssClass} .imhumane-grid {
                background-size: contain;
                background-image: ${challenge.imageUrl};
                display: grid;
                grid-template-columns: repeat(${gridLength}, 1fr);
                grid-template-rows: repeat(${gridLength}, 1fr);
                aspect-ratio: 1;
                box-sizing: border-box;
            }
        `;
    }

    render() {
        const root = this.container.body;
        root.appendChild(this.title);
        root.appendChild(this.grid);
        root.appendChild(this.actions);
    }

    readAnswer() {
        return this.checkboxElements.reduce((prev, elem) =>
            prev + (elem.checked && "1" || "0")
            , "");
    }

    reset() {
        this.checkboxElements.forEach(elem => { elem.checked = false; });
        this.button.disabled = false;
    }

    waitForAnswer() {
        return new Promise((resolve) => {
            this.button.addEventListener("click", () => {
                resolve(this.readAnswer());
            });
        });
    }
}

class ChallengeContainer {
    /**
     *
     * @param {HTMLElement} root
     */
    constructor(root) {
        this.root = root;

        // Add a random string to create a unique class name for styling
        this.cssClass = `imhumane-${Math.random().toString(16).slice(2)}`;

        this.cssStyle = `
            .${this.cssClass} {
                position: relative;
                display: inline-block;
                background: rgb(180,180,200);
                background: linear-gradient(180deg, rgba(180,180,200,1) 0%, rgba(220,240,220,1) 100%);
            }

            .${this.cssClass}.imhumane-responsive div {
                width: 100%;
            }

            .${this.cssClass}, .${this.cssClass} .imhumane-overlay {
                min-height: 4em;
                min-width: 8em;
                z-index: 6;
            }

            .${this.cssClass} .imhumane-overlay {
                position: absolute;
                top: 0;
                left: 0;
                width: 100%;
                height: 100%;
                opacity: 80%;
                background-color: black;
                color: white;
                font-size: 2em;
                text-align: center;
                justify-content: center;
                align-items: center;
            }

            .${this.cssClass} .imhumane-overlay:not([hidden]) {
                display: flex;
            }

            .${this.cssClass}.imhumane-success .imhumane-overlay {
                background-color: darkolivegreen;
            }

            .${this.cssClass} .imhumane-checkbox {
                cursor: pointer;
                opacity: 75%;
                margin: 0;
                padding: 0;
                border: 0;
                z-index: 3;
            }

            .${this.cssClass} .imhumane-checkbox:not(:checked, :hover) {
                appearance: none;
            }

            .${this.cssClass} .imhumane-title {
                box-sizing: border-box;
                margin: 0 0 0 0;
                border: 0;
                line-height: 1.5em;
                padding-top: 1em;
            }

            .${this.cssClass} .imhumane-title b {
                font-size: 2em;
            }

            .${this.cssClass} .imhumane-actions {
                display: block;
                box-sizing: border-box;
            }

            .${this.cssClass} .imhumane-action-submit {
                width: 100%;
                height: 2em;
            }
        `;

        // Elements
        this.style = document.createElement("style");
        this.style.innerHTML = this.cssStyle;

        this.overlay = newElement("div", "imhumane-overlay");

        this.body = newElement("div", "imhumane-body");

        this.tokenInput = newElement("input", "imhumane-token");
        this.tokenInput.type = "hidden";
        this.tokenInput.name = "imhumane_token";
        this.tokenInput.required = true;

        this.doneSetup = false;
    }

    setOverlayText(text) {
        this.overlay.hidden = false;
        this.overlay.innerText = text;
    }

    hideOverlay() {
        this.overlay.hidden = true;
    }

    setup() {
        if (this.doneSetup) return;

        document.head.appendChild(this.style);
        this.root.classList.add(this.cssClass);
        this.root.classList.remove("imhumane-success");
        this.root.appendChild(this.tokenInput);
        this.root.appendChild(this.overlay);
        this.root.appendChild(this.body);

        this.doneSetup = true;
    }

    async runUntilComplete() {
        this.setup();

        while (true) {
            this.setOverlayText("Loading");
            const challenge = await fetchChallenge();
            const grid = new ChallengeGrid(this, challenge);

            // Add the grid's CSS to the existing style element.
            this.style.innerHTML = this.cssStyle + grid.cssStyle;

            grid.render();
            this.hideOverlay();
            const answer = await grid.waitForAnswer();

            this.setOverlayText("Validating");
            try {
                const result = await challenge.validate(answer);
                if (result) {
                    this.setOverlayText("Success!");
                    this.root.classList.add("imhumane-success");
                    this.tokenInput.value = challenge.challengeId;

                    this.root.dispatchEvent(new CustomEvent("imhumane-success", {
                        detail: { token: challenge.challengeId }
                    }));

                    return;
                } else {
                    this.setOverlayText("Failed: Incorrect selection");
                }
            } catch (err) {
                this.setOverlayText(`Validation failed: ${err}`);
            }

            await asleep(3000);
            this.body.innerHTML = "";
            this.style.innerHTML = this.cssStyle;
        }
    }
}

document.addEventListener("DOMContentLoaded", () => {
    document.querySelectorAll(".imhumane-challenge").forEach(async (root) => {
        const challenge = new ChallengeContainer(root);
        await challenge.runUntilComplete();
    });
});
