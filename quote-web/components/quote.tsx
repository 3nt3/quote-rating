import React from "react";
import { NextFunctionComponent } from "next";
import styles from "../styles/Quote.module.scss";
import Image from "next/image";
import { Quote as QuoteModel } from "../models/models";
import dayjs from "dayjs";
import "dayjs/locale/de";
import ReactMarkdown from "react-markdown";

type Props = {
  quote: QuoteModel;
};

function contentWithoutAuthor(content: string): string {
  const lines = contentLines(content);
  return lines.slice(0, -1).join("\n");
}

function contentLines(content: string): string[] {
  return content.split("\n");
}

export default function Quote(props: Props): NextFunctionComponent {
  console.log(props.quote.avatar);
  return (
    <div className={styles.quote}>
      <picture>
        <img
          className={styles.quoteImg}
          src={
            props.quote.avatar ? props.quote.avatar.src : props.quote.avatar_url
          }
          alt="profile pic"
        />
      </picture>
      <div className={styles.quoteText}>
        <p className={styles.quoteUser}>
          {props.quote.username}{" "}
          <span className={styles.quoteDate}>
            {dayjs(props.quote.sent_at).format("DD.MM.YYYY hh:mm:ss")}
          </span>
        </p>
        <div className={styles.quoteQuote}>
          <ReactMarkdown
            components={{
              blockquote: ({ node, ...props }) => (
                <blockquote {...props} className={styles.blockquote}>
                  <div className={styles.blockQuoteThingy}></div>
                  <div>{props.children}</div>
                </blockquote>
              ),
            }}
          >
            {props.quote.content.replace(/([-—–]|--)/, "\n\n\n—")}
          </ReactMarkdown>
        </div>
      </div>
    </div>
  );
}
