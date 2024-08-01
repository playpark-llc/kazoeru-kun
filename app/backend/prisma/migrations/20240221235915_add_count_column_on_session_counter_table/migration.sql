/*
  Warnings:

  - A unique constraint covering the columns `[sessionId]` on the table `SessionCounter` will be added. If there are existing duplicate values, this will fail.
  - Added the required column `count` to the `SessionCounter` table without a default value. This is not possible if the table is not empty.

*/
-- AlterTable
ALTER TABLE `SessionCounter` ADD COLUMN `count` INTEGER NOT NULL;

-- CreateIndex
CREATE UNIQUE INDEX `SessionCounter_sessionId_key` ON `SessionCounter`(`sessionId`);
