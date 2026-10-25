# A first revision pass

The repository includes a three-chapter [sample novel](https://github.com/writerslogic/bluepencil/tree/main/examples/novel). It is a product demo, not a customer case study. From `examples/novel`, run:

```sh
bluepencil report --html report.html
```

The report starts with the passages to review. For example, chapter one places **“gravel” twice within four words** in the sentence about Mara walking up the drive. The writer might remove the second use, choose a more precise image, or keep the repeated sound of the word. `bluepencil echoes` also points to a run of **“looked”** in the motel scene; that repetition may suit the old man's watchful behavior.

The sample currently contains 345 prose words, 16 echo observations, and 7 adverb observations. These are measured counts, not a grade. `bluepencil check` applies the sample's configured thresholds; the optional model-backed commands are separate and may send text to your configured provider.

If you try bluepencil on your own work, use the [writer feedback form](https://github.com/writerslogic/bluepencil/issues/new?template=writer_feedback.yml) to share what helped and what got in the way. Include a short passage only if you may post it publicly. A publishable case study needs the writer's explicit consent; none is implied by an issue or a star.
