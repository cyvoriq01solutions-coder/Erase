import { Route, Routes } from "react-router";
import PublicLayout from "./layouts/PublicLayout";
import AppLayout from "./layouts/AppLayout";
import HomePage from "./pages/HomePage";
import InfoPage from "./pages/InfoPage";
import DashboardPage from "./pages/DashboardPage";
import NotFoundPage from "./pages/NotFoundPage";

export default function App() {
  return (
    <Routes>
      <Route element={<PublicLayout />}>
        <Route path="/" element={<HomePage />} />
        <Route path="/platform" element={<InfoPage title="Platform" />} />
        <Route path="/how-it-works" element={<InfoPage title="How It Works" />} />
        <Route path="/assurance" element={<InfoPage title="Assurance" />} />
        <Route path="/security" element={<InfoPage title="Security" />} />
        <Route path="/resources" element={<InfoPage title="Resources" />} />
        <Route path="/contact" element={<InfoPage title="Contact" />} />
      </Route>

      <Route path="/app" element={<AppLayout />}>
        <Route index element={<DashboardPage />} />
        <Route path="dashboard" element={<DashboardPage />} />
        <Route path="devices" element={<InfoPage title="Devices" />} />
        <Route path="assessments" element={<InfoPage title="Assessments" />} />
        <Route path="evidence" element={<InfoPage title="Evidence" />} />
        <Route path="verification" element={<InfoPage title="Verification" />} />
        <Route path="reports" element={<InfoPage title="Reports" />} />
        <Route path="certificates" element={<InfoPage title="Certificates" />} />
        <Route path="settings" element={<InfoPage title="Settings" />} />
      </Route>

      <Route path="*" element={<NotFoundPage />} />
    </Routes>
  );
}
