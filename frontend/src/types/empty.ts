// @generated by protobuf-ts 2.9.4
// @generated from protobuf file "empty.proto" (syntax proto2)
// tslint:disable
import type { BinaryWriteOptions } from "@protobuf-ts/runtime";
import type { IBinaryWriter } from "@protobuf-ts/runtime";
import { UnknownFieldHandler } from "@protobuf-ts/runtime";
import type { BinaryReadOptions } from "@protobuf-ts/runtime";
import type { IBinaryReader } from "@protobuf-ts/runtime";
import type { PartialMessage } from "@protobuf-ts/runtime";
import { reflectionMergePartial } from "@protobuf-ts/runtime";
import { MessageType } from "@protobuf-ts/runtime";
/**
 * @generated from protobuf message EmptyMessage
 */
export interface EmptyMessage {
}
// @generated message type with reflection information, may provide speed optimized methods
class EmptyMessage$Type extends MessageType<EmptyMessage> {
    constructor() {
        super("EmptyMessage", []);
    }
    create(value?: PartialMessage<EmptyMessage>): EmptyMessage {
        const message = globalThis.Object.create((this.messagePrototype!));
        if (value !== undefined)
            reflectionMergePartial<EmptyMessage>(this, message, value);
        return message;
    }
    internalBinaryRead(reader: IBinaryReader, length: number, options: BinaryReadOptions, target?: EmptyMessage): EmptyMessage {
        return target ?? this.create();
    }
    internalBinaryWrite(message: EmptyMessage, writer: IBinaryWriter, options: BinaryWriteOptions): IBinaryWriter {
        let u = options.writeUnknownFields;
        if (u !== false)
            (u == true ? UnknownFieldHandler.onWrite : u)(this.typeName, message, writer);
        return writer;
    }
}
/**
 * @generated MessageType for protobuf message EmptyMessage
 */
export const EmptyMessage = new EmptyMessage$Type();
