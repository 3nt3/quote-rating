export interface Quote {
	id: number;
  content: string;
  author_id: number;
  created_at: number;
  sent_at: number;
  username: string;
  score: number;
  channel_id: number;
  message_id: number;
  message_link: string;
  avatar_url: string;
}
