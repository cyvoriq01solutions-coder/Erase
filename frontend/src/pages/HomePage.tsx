import { NavLink } from "react-router";

const stages = ["ASSESS", "PDEM", "EVIDENCE", "VERIFY", "REPORT"];

export default function HomePage() {
  return (
    <main>
      <section className="hero">
        <div className="hero-copy">
          <span className="eyebrow">Independent Device Data-Sanitization Assurance</span>
          <h1>Verify the evidence before you trust the outcome.</h1>
          <p>
            CYVORIQ Erase is being engineered as an assurance platform for retired enterprise devices,
            separating assessment, evidence, verification and reporting from destructive execution.
          </p>
          <div className="hero-actions">
            <NavLink className="button" to="/platform">Explore Platform</NavLink>
            <NavLink className="button button-secondary" to="/assurance">View Assurance Model</NavLink>
          </div>
        </div>
        <div className="assurance-card" aria-label="MVP verification lifecycle">
          <span className="status-pill">MVP · NON-DESTRUCTIVE</span>
          <h2>Verification first.</h2>
          <p>The initial Windows Verification Agent will not perform destructive sanitization.</p>
          <div className="stage-grid">
            {stages.map((stage, index) => (
              <div className="stage" key={stage}>
                <span>{String(index + 1).padStart(2, "0")}</span>
                <strong>{stage}</strong>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="principles">
        <article>
          <span className="section-number">01</span>
          <h3>Security</h3>
          <p>Clear trust boundaries between browser, control plane, database and future device agents.</p>
        </article>
        <article>
          <span className="section-number">02</span>
          <h3>Evidence</h3>
          <p>Structured, traceable evidence designed to support independent verification.</p>
        </article>
        <article>
          <span className="section-number">03</span>
          <h3>Engineering</h3>
          <p>Exact dependency versions, reproducible environments and staged capability releases.</p>
        </article>
      </section>
    </main>
  );
}
