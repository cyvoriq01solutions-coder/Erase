const cards = [
  ["Devices", "0"],
  ["Assessments", "0"],
  ["Evidence records", "0"],
  ["Verification results", "0"],
];

export default function DashboardPage() {
  return (
    <section>
      <div className="page-heading">
        <span className="eyebrow">CONTROL PLANE</span>
        <h1>Dashboard</h1>
        <p>Frontend shell only. No production API or database connection is enabled in this package.</p>
      </div>
      <div className="metric-grid">
        {cards.map(([label, value]) => (
          <article className="metric-card" key={label}>
            <span>{label}</span>
            <strong>{value}</strong>
          </article>
        ))}
      </div>
    </section>
  );
}
