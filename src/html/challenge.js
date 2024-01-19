// Requires IMHUMANE_API_URL to be defined earlier
// TODO serve via API
// TODO validate button + set valid token
class Challenge {
    constructor(
        headers,
        base64Image,
    ) {
        this.challengeId = headers.get("X-Imhumane-Id");
        this.topic = headers.get("X-Imhumane-Topic");
        this.gapSize = +headers.get("X-Imhumane-Gap-Size");
        this.imageSize = +headers.get("X-Imhumane-Image-Size");
        this.gridLength = +headers.get("X-Imhumane-Grid-Length");
        this.containerLength = this.gapSize + ((this.gapSize + this.imageSize) * this.gridLength);

        this.imageUrl = `url("${base64Image}")`;

        this.answerElement = this.getAnswerElement();
        this.checkboxElements = [];
        for (let i = 1; i <= this.gridLength ** 2; i++) {
            this.checkboxElements.push(this.getCheckboxElement(i));
        }
    }

    get cssClass() {
        return `imhumane-${this.challengeId}`
    }

    get cssStyle() {
        return `
            .${this.cssClass} > div {
                background-image: ${this.imageUrl};
                display: grid;
                grid-template-columns: repeat(${this.gridLength}, ${this.imageSize}px);
                grid-template-rows: repeat(${this.gridLength}, ${this.imageSize}px);
                gap: ${this.gapSize}px;
                padding: ${this.gapSize}px;
                height: ${this.containerLength}px;
                width: ${this.containerLength}px;
                box-sizing: border-box;
            }

            .${this.cssClass} > div > input {
                cursor: pointer;
                opacity: 75%;
                margin: 0;
                padding: 0;
                border: 0;
            }

            .${this.cssClass} > div > input:not(:checked) {
                appearance: none;
            };
        `;
    }

    getStyleElement() {
        const style = document.createElement("style");
        style.innerHTML = this.cssStyle;
        return style;
    }

    getCheckboxElement(index) {
        const checkbox = document.createElement("input");
        checkbox.type = "checkbox";
        checkbox.onchange = () => {
            this.answerElement.value = this.readAnswer();
        }
        return checkbox;
    }

    getImageContainerElement() {
        const selector = document.createElement("div");
        selector.classList.add("imhumane-image");
        return selector;
    }

    getAnswerElement() {
        const answer = document.createElement("input");
        topic.classList.add("imhumane-answer");
        answer.type = "hidden";
        answer.name = "imhumane-answer";
        return answer
    }

    getTopicElement() {
        const topic = document.createElement("p");
        topic.classList.add("imhumane-topic");
        topic.innerHTML = `Select all images containing <b>${this.topic}</b>:`;
        return topic
    }

    /**
     *
     * @param {HTMLElement} root
     */
    configureElement(root) {
        root.classList.add(this.cssClass);

        const imgContainer = this.getImageContainerElement();
        this.checkboxElements.forEach(elem => imgContainer.appendChild(elem));

        root.appendChild(this.getTopicElement());
        root.appendChild(imgContainer);
        root.appendChild(this.answerElement);
    }

    readAnswer() {
        return this.checkboxElements.reduce((prev, elem) =>
            prev + (elem.checked && "1" || "0")
            , "");
    }
}

// From SO: https://stackoverflow.com/a/18650249
function blobToBase64(blob) {
    return new Promise((resolve, _) => {
        const reader = new FileReader();
        reader.onloadend = () => resolve(reader.result);
        reader.readAsDataURL(blob);
    });
}

async function configureChallenge(root) {
    // Fetch a challenge
    const response = await fetch(`${IMHUMANE_API_URL}`, {
        method: "GET",
    });
    const image = await blobToBase64(await response.blob());

    const challenge = new Challenge(response.headers, image);
    document.head.appendChild(
        challenge.getStyleElement()
    );
    challenge.configureElement(root);
}

document.addEventListener("DOMContentLoaded", () => {
    document.querySelectorAll(".imhumane-challenge").forEach((root) => configureChallenge(root));
});