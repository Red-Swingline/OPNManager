// Interface related types
export interface IpAddress {
  ipaddr: string;
}

export interface VlanInfo {
  tag: string;
  proto: string;
  pcp: string;
  parent: string;
}

export interface Interface {
  flags: string[];
  capabilities: string[];
  options: string[];
  macaddr: string;
  supported_media: string[];
  is_physical: boolean;
  device: string;
  mtu: string;
  macaddr_hw?: string;
  media?: string;
  media_raw?: string;
  status: string;
  routes: string[];
  config?: any;
  groups: string[];
  vlan?: VlanInfo;
  identifier: string;
  description: string;
  enabled: boolean;
  link_type?: string;
  addr4?: string;
  addr6?: string;
  ipv4: IpAddress[];
  ipv6: IpAddress[];
  vlan_tag?: string;
  gateways: string[];
}

// Device related types
export interface CombinedDevice {
  mac: string;
  ipv4_addresses: string[];
  ipv6_addresses: string[];
  intf: string;
  expired?: boolean;
  expires?: number;
  permanent?: boolean;
  device_type?: string;
  manufacturer: string;
  hostname: string;
  intf_description: string;
}

// Topology specific types
export interface TopologyNode {
  id: string;
  type: 'interface' | 'device';
  label: string;
  data: Interface | CombinedDevice;
  x?: number;
  y?: number;
}

export interface TopologyLink {
  source: string;
  target: string;
  id: string;
}

export interface TopologyData {
  nodes: TopologyNode[];
  links: TopologyLink[];
}