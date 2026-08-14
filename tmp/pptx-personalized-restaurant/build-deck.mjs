import fs from "node:fs/promises";
import path from "node:path";
import { Presentation, PresentationFile } from "@oai/artifact-tool";

const imageDir = String.raw`C:\Users\user\Documents\Development_Project\Rust Project\fyp_recommender\output\ppt\personalized-restaurant-ordering-system\页面图片`;
const outputPath = String.raw`C:\Users\user\Documents\Development_Project\Rust Project\fyp_recommender\output\ppt\personalized-restaurant-ordering-system\Personalized-Restaurant-Ordering-System.pptx`;
const previewDir = String.raw`C:\Users\user\Documents\Development_Project\Rust Project\fyp_recommender\tmp\pptx-personalized-restaurant\artifact-previews`;

const notes = [
  `Open by positioning the project as a mobile-first ordering and recommendation prototype for a small restaurant. Introduce the three recommendation signals and the QR-based access model.\n\n[Sources]\n- FR_22049837.pdf, title page and Abstract\n- Repository mobile customer interface evidence\n[/Sources]`,
  `Explain the four constraints: sparse history, cold start, hard restrictions, and weak transparency. Emphasise that the study evaluates observable recommendation behaviour, not sales or customer satisfaction.\n\n[Sources]\n- FR_22049837.pdf, Sections 1.1–1.2\n- Bondevik et al. (2024), DOI 10.1016/j.eswa.2023.122166\n- Zhang & Chen (2020), DOI 10.1561/1500000066\n[/Sources]`,
  `Walk through RQ1, RQ2, and RQ3 as three controlled evaluation paths. Connect them to the grouped objectives: build the artefact, implement explainable recommendation, and evaluate observable behaviour.\n\n[Sources]\n- FR_22049837.pdf, Sections 1.3–1.4\n[/Sources]`,
  `Describe the customer journey from QR entry to checkout and the staff journey from Pending to Completed. Explain that only a completed basket becomes future historical recommendation evidence.\n\n[Sources]\n- FR_22049837.pdf, project scope and workflow sections\n- docs/report-evidence/2026-07-29/figure-4-6a-customer-order-submitted.png\n- docs/report-evidence/2026-07-29/figure-4-6b-admin-live-order.png\n- docs/report-evidence/2026-07-29/figure-4-6c-customer-status-updated.png\n[/Sources]`,
  `Explain the single-process deployment and the internal responsibility boundaries. Stress that routes, scoring, persistence, and presentation are separated even though the prototype remains lightweight.\n\n[Sources]\n- FR_22049837.pdf, Section 3.4\n- docs/ARCHITECTURE.md\n[/Sources]`,
  `Start with the hard eligibility gate: unavailable, selected, and disliked-ingredient dishes are removed before ranking. Then explain component scoring, adaptive weighting, deterministic ranking, diversity, and explanation generation.\n\n[Sources]\n- FR_22049837.pdf, recommendation design and implementation sections\n- docs/ARCHITECTURE.md, Recommendation Flow\n[/Sources]`,
  `Read the recommendation reason as an evidence trace. Distinguish ranking score from evidence confidence, and state clearly that confidence is not a probability that the customer will buy or like the dish.\n\n[Sources]\n- FR_22049837.pdf, Sections 1.6, 3.4, and 5.3\n- Zhang & Chen (2020), DOI 10.1561/1500000066\n[/Sources]`,
  `Summarise the implemented customer, administrator, and evaluation capabilities. Keep the claim within the single-restaurant local prototype scope.\n\n[Sources]\n- FR_22049837.pdf, Chapter 4\n- docs/report-evidence/2026-07-29/figure-4-7-mobile-390x844.png\n- docs/report-evidence/2026-07-29/figure-4-6b-admin-live-order.png\n- docs/report-evidence/2026-07-29/section-4-4/figure-4-12-method-comparison-interface.png\n[/Sources]`,
  `Keep the two measurements separate: 115 Rust implementation tests and 55 selected report-level requirement checks. Explain the category breakdown and the controlled local test boundary.\n\n[Sources]\n- FR_22049837.pdf, Section 4.3\n- docs/report-evidence/2026-07-29/SYSTEM_TESTING_RESULTS.md\n[/Sources]`,
  `Use the rank changes to answer RQ1 and RQ2. Liked ingredients raised compatible dishes, disliked ingredients caused hard exclusion, and stronger co-order evidence could improve rank. Note that D09 stayed first because it was already at the top.\n\n[Sources]\n- FR_22049837.pdf, Sections 4.4.1–4.4.2\n- docs/report-evidence/2026-07-29/section-4-4/ingredient-impact-results.csv\n- docs/report-evidence/2026-07-29/section-4-4/coorder-impact-results.csv\n[/Sources]`,
  `Lead with Hit@3: 20 percent for ingredient-only and 100 percent for co-order-only and fixed hybrid. Then show average rank and immediately state the five-case, fixed-profile experimental boundary.\n\n[Sources]\n- FR_22049837.pdf, Section 4.4.3\n- docs/report-evidence/2026-07-29/section-4-4/method-comparison-results.csv\n- docs/report-evidence/2026-07-29/section-4-4/method-comparison-summary.csv\n[/Sources]`,
  `Balance the achieved technical contribution with the explicit research limitations. Connect each limitation to a concrete validation or engineering next step.\n\n[Sources]\n- FR_22049837.pdf, Sections 1.5, 5.3, and 5.4\n- Bondevik et al. (2024), DOI 10.1016/j.eswa.2023.122166\n[/Sources]`,
  `Restate the central conclusion in one sentence and transition directly to the four-step live demonstration. Keep questions until after the demonstration.\n\n[Sources]\n- FR_22049837.pdf, Chapter 5\n- Implemented customer and administrator interfaces in this repository\n[/Sources]`,
];

async function readArrayBuffer(filePath) {
  const bytes = await fs.readFile(filePath);
  return bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength);
}

async function writeBlob(filePath, blob) {
  await fs.mkdir(path.dirname(filePath), { recursive: true });
  await fs.writeFile(filePath, new Uint8Array(await blob.arrayBuffer()));
}

async function main() {
  await fs.mkdir(path.dirname(outputPath), { recursive: true });
  await fs.mkdir(previewDir, { recursive: true });

  const presentation = Presentation.create({
    slideSize: { width: 1280, height: 720 },
  });

  for (let page = 1; page <= 13; page += 1) {
    const pageLabel = String(page).padStart(2, "0");
    const imagePath = path.join(imageDir, `第${pageLabel}页.png`);
    const slide = presentation.slides.add();
    slide.images.add({
      blob: await readArrayBuffer(imagePath),
      contentType: "image/png",
      alt: `Personalized Restaurant Ordering System, slide ${page} of 13`,
      fit: "cover",
      position: { left: 0, top: 0, width: 1280, height: 720 },
    });
    slide.speakerNotes.textFrame.setText(notes[page - 1]);
    slide.speakerNotes.setVisible(true);

    const preview = await presentation.export({ slide, format: "png", scale: 1 });
    await writeBlob(path.join(previewDir, `slide-${pageLabel}.png`), preview);
  }

  const montage = await presentation.export({ format: "webp", montage: true, scale: 1 });
  await writeBlob(path.join(previewDir, "deck-montage.webp"), montage);

  const pptx = await PresentationFile.exportPptx(presentation);
  await pptx.save(outputPath);
  console.log(outputPath);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
