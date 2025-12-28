import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { useAuth } from './hooks/useAuth';
import { Login } from './pages/Login';
import { Projects } from './pages/Projects';
import { Environments } from './pages/Environments';
import { Flags } from './pages/Flags';
import { FlagEditor } from './pages/FlagEditor';

function PrivateRoute({ children }: { children: React.ReactNode }) {
  const { isAuthenticated } = useAuth();
  return isAuthenticated ? <>{children}</> : <Navigate to="/login" />;
}

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/login" element={<Login />} />
        <Route
          path="/projects"
          element={
            <PrivateRoute>
              <Projects />
            </PrivateRoute>
          }
        />
        <Route
          path="/projects/:projectId/environments"
          element={
            <PrivateRoute>
              <Environments />
            </PrivateRoute>
          }
        />
        <Route
          path="/projects/:projectId/environments/:environmentId/flags"
          element={
            <PrivateRoute>
              <Flags />
            </PrivateRoute>
          }
        />
        <Route
          path="/projects/:projectId/environments/:environmentId/flags/:flagId"
          element={
            <PrivateRoute>
              <FlagEditor />
            </PrivateRoute>
          }
        />
        <Route path="*" element={<Navigate to="/projects" />} />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
