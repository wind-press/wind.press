
const options = {};

let notifier = new AWN(options);

async function dropHandler(ev) {
    console.log("File(s) dropped");

    // Prevent default behavior (Prevent file from being opened)
    ev.preventDefault();

    if (ev.dataTransfer.items) {
        // Use DataTransferItemList interface to access the file(s)
        [...ev.dataTransfer.items].forEach((item, i) => {
            // If dropped items aren't files, reject them
            if (item.kind === "file") {
                const file = item.getAsFile();
                console.log(`… file[${i}].name = ${file.name}`);

                // send "multipart/form-data" POST request to server, with origin set to "https://wakufont-hosted.rosua.org"
                const formData = new FormData();
                formData.append("file", file);

                const promise = fetch("https://wakufont-hosted.rosua.org/tool/google/woff2/compress", {
                    method: "POST",
                    body: formData,
                    mode: "cors",
                })

                    // server is returning a binary file, so we need to tell the browser to download it
                    .then(async res => {
                        // get the response body as a stream
                        const reader = res.body.getReader();

                        // read the stream until it's done
                        const chunks = [];
                        while (true) {
                            const { done, value } = await reader.read();
                            if (done) break;
                            chunks.push(value);
                        }

                        // concatenate the chunks into a single Uint8Array
                        const bytes = new Uint8Array(chunks.reduce((acc, val) => acc + val.length, 0));
                        let offset = 0;
                        for (const chunk of chunks) {
                            bytes.set(chunk, offset);
                            offset += chunk.length;
                        }

                        // convert the Uint8Array to a Blob
                        const blob = new Blob([bytes], { type: res.headers.get("Content-Type") });

                        // get the filename from the file variable (which is the original file name), and replace the extension with ".woff2"
                        const filename = file.name.replace(/\.[^/.]+$/, "") + ".woff2";

                        // create a link to download the file
                        const a = document.createElement("a");
                        a.href = URL.createObjectURL(blob);
                        a.download = filename;
                        a.click();
                    });

                new AWN().async(
                    promise,
                    `Compressed: ${file.name}`,
                    `Failed: ${file.name}`,
                    `Compressing file: ${file.name} ...`,
                );
            }
        });
    } else {
        // Use DataTransfer interface to access the file(s)
        [...ev.dataTransfer.files].forEach((file, i) => {
            console.log(`… file[${i}].name = ${file.name}`);
        });
    }
}

function dragOverHandler(ev) {
    // Prevent default behavior (Prevent file from being opened)
    ev.preventDefault();
}