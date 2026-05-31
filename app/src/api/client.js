const API_BASE = '/api/v1';

export async function fetchApi(endpoint, options = {}) {
    const headers = { 
        ...options.headers,
    };

    if (!(options.body instanceof FormData) && !headers['Content-Type']) {
        headers['Content-Type'] = 'application/json';
    }

    const url = `${API_BASE}${endpoint}`;
    
    const res = await fetch(url, {
        ...options,
        headers,
        credentials: 'include',
    });

    if (res.status === 204) return null;

    const text = await res.text();
    
    let json;
    try {
        json = JSON.parse(text);
    } catch (e) {
        console.error(`[API] JSON parse error:`, e, 'Response was:', text);
        throw new Error(text || `Failed to parse JSON response: ${res.status} ${res.statusText}`);
    }

    if (!json.success && json.success !== undefined) {
        console.error(`[API] Error response:`, json);
        throw new Error(json?.error?.message || json?.message || 'API Error');
    }
    return json.data ?? json;
}

export function connectWebSocket(serverIds = []) {
    const params = serverIds.length
        ? `?servers=${serverIds.join(',')}`
        : '';
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const ws = new WebSocket(`${protocol}//${window.location.host}/ws${params}`);
    return ws;
}

export async function generateNodeKey(nodeId, name = null) {
    return fetchApi(`/nodes/${nodeId}/generate-key`, {
        method: 'POST',
        body: JSON.stringify({ name })
    });
}

export async function getNodes() {
    return fetchApi('/nodes');
}

export async function getNode(nodeId) {
    return fetchApi(`/nodes/${nodeId}`);
}

export async function createNode(data) {
    return fetchApi('/nodes', { method: 'POST', body: JSON.stringify(data) });
}

export async function updateNode(nodeId, data) {
    return fetchApi(`/nodes/${nodeId}`, { method: 'PUT', body: JSON.stringify(data) });
}

export async function deleteNode(nodeId) {
    return fetchApi(`/nodes/${nodeId}`, { method: 'DELETE' });
}

export async function getNodeKeys(nodeId) {
    return fetchApi(`/nodes/${nodeId}/keys`);
}

export async function deleteApiKey(nodeId, keyId) {
    return fetchApi(`/nodes/${nodeId}/keys/${keyId}`, {
        method: 'DELETE'
    });
}

export async function getNodeMetrics(nodeId) {
    return fetchApi(`/nodes/${nodeId}/metrics`);
}

export async function getNodeHealth(nodeId) {
    return fetchApi(`/nodes/${nodeId}/health`);
}

export async function generateRegistrationToken(nodeId, expiresInHours = 24) {
    return fetchApi(`/nodes/${nodeId}/tokens`, {
        method: 'POST',
        body: JSON.stringify({ expires_in_hours: expiresInHours })
    });
}

export async function getRegistrationTokens(nodeId) {
    return fetchApi(`/nodes/${nodeId}/tokens`);
}

export async function revokeRegistrationToken(nodeId, tokenId) {
    return fetchApi(`/nodes/${nodeId}/tokens/${tokenId}`, {
        method: 'DELETE'
    });
}

export async function getRefundEligibility(subscriptionId) {
    return fetchApi(`/billing/refund/eligibility?subscription_id=${subscriptionId}`);
}

export async function requestRefund(subscriptionId, reason = '') {
    return fetchApi('/billing/refund', {
        method: 'POST',
        body: JSON.stringify({ subscription_id: subscriptionId, reason })
    });
}

export async function getRefunds() {
    return fetchApi('/billing/refunds');
}

// ─── Scheduled Actions (Phase 59) ───
export async function getSchedules(serverId) {
    return fetchApi(`/servers/${serverId}/tasks`);
}

export async function createSchedule(serverId, data) {
    return fetchApi(`/servers/${serverId}/tasks`, {
        method: 'POST',
        body: JSON.stringify(data),
    });
}

export async function updateSchedule(serverId, taskId, data) {
    return fetchApi(`/servers/${serverId}/tasks/${taskId}`, {
        method: 'PATCH',
        body: JSON.stringify(data),
    });
}

export async function deleteSchedule(serverId, taskId) {
    return fetchApi(`/servers/${serverId}/tasks/${taskId}`, { method: 'DELETE' });
}

// ─── Crash History (Phase 60) ───
export async function getCrashLogs(serverId, limit = 20, offset = 0) {
    return fetchApi(`/servers/${serverId}/crash-logs?limit=${limit}&offset=${offset}`);
}

export async function clearCrashLogs(serverId) {
    return fetchApi(`/servers/${serverId}/crash-logs`, { method: 'DELETE' });
}

export async function acknowledgeCrash(serverId, logId) {
    return fetchApi(`/servers/${serverId}/crash-logs/${logId}/resolve`, { method: 'POST' });
}
