type InfoPageProps = { title: string };

export default function InfoPage({ title }: InfoPageProps) {
  return (
    <section className="content-page">
      <span className="eyebrow">CYVORIQ ERASE</span>
      <h1>{title}</h1>
      <p>This route is established as part of the frozen frontend architecture. Product content will be added in the appropriate build package.</p>
    </section>
  );
}
