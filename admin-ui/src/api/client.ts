import axios from 'axios';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';

const api = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Add auth token to requests
api.interceptors.request.use((config) => {
  const token = localStorage.getItem('token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// Handle 401 responses
api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('token');
      window.location.href = '/login';
    }
    return Promise.reject(error);
  }
);

// Auth
export const login = (email: string, password: string) =>
  api.post<{ token: string }>('/auth/login', { email, password });

export const register = (email: string, password: string) =>
  api.post<{ token: string }>('/auth/register', { email, password });

// Projects
export const getProjects = () => api.get<Project[]>('/api/projects');
export const createProject = (name: string, description?: string) =>
  api.post<Project>('/api/projects', { name, description });
export const deleteProject = (id: string) => api.delete(`/api/projects/${id}`);

// Environments
export const getEnvironments = (projectId: string) =>
  api.get<Environment[]>(`/api/projects/${projectId}/environments`);
export const createEnvironment = (projectId: string, name: string, key: string, description?: string) =>
  api.post<Environment>(`/api/projects/${projectId}/environments`, { name, key, description });

// Flags
export const getFlags = (projectId: string, environmentId: string) =>
  api.get<Flag[]>(`/api/projects/${projectId}/environments/${environmentId}/flags`);
export const createFlag = (projectId: string, environmentId: string, key: string, name: string, description?: string) =>
  api.post<Flag>(`/api/projects/${projectId}/environments/${environmentId}/flags`, { key, name, description });
export const updateFlag = (projectId: string, environmentId: string, flagId: string, data: Partial<Flag>) =>
  api.put<Flag>(`/api/projects/${projectId}/environments/${environmentId}/flags/${flagId}`, data);
export const toggleFlag = (projectId: string, environmentId: string, flagId: string) =>
  api.post<Flag>(`/api/projects/${projectId}/environments/${environmentId}/flags/${flagId}/toggle`);
export const deleteFlag = (projectId: string, environmentId: string, flagId: string) =>
  api.delete(`/api/projects/${projectId}/environments/${environmentId}/flags/${flagId}`);

// Rules
export const getRules = (projectId: string, environmentId: string, flagId: string) =>
  api.get<Rule[]>(`/api/projects/${projectId}/environments/${environmentId}/flags/${flagId}/rules`);
export const createRule = (projectId: string, environmentId: string, flagId: string, data: CreateRuleData) =>
  api.post<Rule>(`/api/projects/${projectId}/environments/${environmentId}/flags/${flagId}/rules`, data);
export const deleteRule = (projectId: string, environmentId: string, flagId: string, ruleId: string) =>
  api.delete(`/api/projects/${projectId}/environments/${environmentId}/flags/${flagId}/rules/${ruleId}`);

// Types
export interface Project {
  id: string;
  name: string;
  description: string | null;
  sdk_key: string;
  created_at: string;
  updated_at: string;
}

export interface Environment {
  id: string;
  project_id: string;
  name: string;
  key: string;
  description: string | null;
  created_at: string;
  updated_at: string;
}

export interface Flag {
  id: string;
  environment_id: string;
  key: string;
  name: string;
  description: string | null;
  enabled: boolean;
  rollout_percentage: number;
  created_at: string;
  updated_at: string;
}

export interface Rule {
  id: string;
  flag_id: string;
  rule_type: string;
  rule_value: string;
  enabled: boolean;
  priority: number;
  created_at: string;
  updated_at: string;
}

export interface CreateRuleData {
  rule_type: string;
  rule_value: string;
  priority?: number;
}

export default api;
