from flask import Flask, request, jsonify
from groq import Groq
from google.protobuf.json_format import MessageToDict
from protos_py import client_state_pb2
from protos_py import player_pb2
import json
import os

groq_api_key = os.getenv('GROQ_API')

client = Groq(api_key=groq_api_key)

app = Flask(__name__)

@app.route('/pocker_move', methods=['POST'])
def pocker_move():
    try:
        client_state = client_state_pb2.ClientState()
        client_state.ParseFromString(request.data)
    except Exception as e:
        return jsonify({'error': 'Failed to parse protobuf message', 'details': str(e)}), 400

    client_state_dict = MessageToDict(client_state)

    messages = [
        {"role": 'user', "content": str(client_state_dict)},
        {"role": "system", "content": """You are a professional poker player. Your task is to analyze the current game situation. In provided dictionary message you will find your id is equal to player_id field. Answer in the following JSON format:
        {
          "action_type": 0|1|2|3,
          "bet": number,
          "explanation": string
        } Where action_type is one of the enums options Fold = 0; Call = 1; Raise = 2; Check = 3; And explanation is why you did that move"""}
    ]

    try:
        response = client.chat.completions.create(
            model='llama3-70b-8192',
            messages=messages,
            temperature=0
        )
    except Exception as e:
        return jsonify({'error': 'Failed to call Groq API', 'details': str(e)}), 500

    response_text = response.choices[0].message.content
    print(response_text)
    response_text_json = json.loads(response_text)

    try:
        action_message = player_pb2.Action()
        action_type = response_text_json.get('action_type')
        bet = response_text_json.get('bet')
        action_message.action_type = int(action_type)
        action_message.bet = int(bet)
    except Exception as e:
        return jsonify({'error': 'Failed to parse action message from response', 'details': str(e)}), 500

    return action_message.SerializeToString(), 200, {'Content-Type': 'application/octet-stream'}

if __name__ == '__main__':
    app.run(port=5000)