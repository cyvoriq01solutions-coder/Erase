import { Route, Routes } from "react-router";
import PublicLayout from "./layouts/PublicLayout";
import HomePage from "./pages/HomePage";
import InfoPage from "./pages/InfoPage";
import DownloadPage from "./pages/DownloadPage";
import AccountPage from "./pages/AccountPage";
import NotFoundPage from "./pages/NotFoundPage";

export default function App() {
  return (
    <Routes>
      <Route element={<PublicLayout />}>
        <Route path="/" element={<HomePage />} />
        <Route path="/why-cyvra" element={<InfoPage title="Why CYVRA" />} />
        <Route path="/how-it-works" element={<InfoPage title="How It Works" />} />
        <Route path="/dpdp-readiness" element={<InfoPage title="DPDP Readiness" />} />
        <Route path="/individuals" element={<InfoPage title="For Individuals" />} />
        <Route path="/enterprise" element={<InfoPage title="Enterprise & OEM" />} />
        <Route path="/resources" element={<InfoPage title="Resources" />} />
        <Route path="/contact" element={<InfoPage title="Contact" />} />
        <Route path="/download" element={<DownloadPage />} />
        <Route path="/account" element={<AccountPage />} />

        <Route path="/platform" element={<InfoPage title="Platform" />} />
        <Route path="/assurance" element={<InfoPage title="Assurance" />} />
        <Route path="/security" element={<InfoPage title="Security" />} />
      </Route>

      <Route path="*" element={<NotFoundPage />} />
    </Routes>
  );
}
